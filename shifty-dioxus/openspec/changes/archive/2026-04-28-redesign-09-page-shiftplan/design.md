## Context

`src/page/shiftplan.rs` is the most complex screen in the application — it stitches together a toolbar (week navigation, view toggle, sales-person picker, copy-from-last-week, iCal exports), a tab bar that selects between shiftplans, the week grid (`WeekView`), a working-hours mini overview, an embedded `SlotEdit` modal, a week-message editor, a shiftplan report panel, and a booking-log table. The current implementation:

- Renders the toolbar with bare `<button>` elements styled with `border-2 border-solid border-black` and `bg-blue-500 text-white` / `bg-gray-200`. The week label sits between two `<` and `>` buttons; the view toggle is two adjacent buttons; the `Edit as` selector is a `<select class="bg-slate-200 p-1 rounded-md">`; the dropdown trigger that holds "New slot" / structure-mode is a `<button class="border-2 border-solid border-black ...">`.
- Uses `src/component/shiftplan_tab_bar.rs` whose tabs carry `border-blue-500 text-blue-600` (active) and `text-gray-500 hover:text-gray-700` (inactive); the `+` create button is `text-green-600` and the per-tab `✕` delete is `text-red-400`. Create/edit/delete dialogs render through the legacy `Modal` (`src/component/modal.rs`), with raw `<input class="...border-gray-300...">` fields and `bg-blue-500` / `bg-red-500` confirm buttons.
- Uses `src/component/week_view.rs` which renders the week as a `flex flex-row` of seven `<DayView>` columns plus a sticky `<TimeView>` time column. Each cell is an absolutely positioned `<ColumnViewSlot>` with hard-coded `border-solid border-black border` and a separate stack of three buttons (`+`, `−`, `…` dropdown) whose visibility depends on `WeekViewButtonTypes`. PersonChips render as `<p>` elements with inline `background-color`, with no per-chip × button (per-chip remove is not implemented today). Min-resources renders as a chip with hard-coded `#ffcccc` (understaffed) and `#fff` (ok). Zoom is implemented via a `scale-down-50 / 75 / 100` Tailwind helper that maps to CSS `zoom`.
- Uses `src/component/working_hours_mini_overview.rs` — a vertical `flex flex-col max-w-96` list with `text-red-800` (under target) and `text-green-800` (at/over target). No progress bar, no color dot.
- Uses `src/component/booking_log_table.rs` — a filter row in `bg-gray-50` over a `min-w-full border-collapse border border-gray-300` table. Deleted rows render with `line-through text-gray-500`. Filter dropdowns use `border-gray-300`; the clear button is `bg-blue-500`.
- Uses `src/component/slot_edit.rs` — a legacy `Modal` containing the legacy `Form`, `FormGroup`, `FormPair`, `Select`, `TimeInput`, `IntegerInput`, and `Button` from `base_components.rs`.

The reference design (`design_handoff_shifty/README.md` § 2) reframes the page around four visual surfaces:

1. **Toolbar** — Prev/next-week `NavBtn`s with the week label `KW <n> · <date range>` between them, a Woche/Tag segmented toggle inside a `bg-surface-alt` pill, action buttons (`Letzte Woche`, iCal exports) styled as `Btn` Secondary/Ghost, and a `Du bearbeitest:` select using the new `Field` + `FormSelectInput` form atoms.
2. **Tab bar** — flat underline tabs (`border-b-2 border-accent text-accent` active / `border-b-2 border-transparent text-ink-soft` inactive) with the structure-mode `+` and `✕` controls restyled on tokens. Create/edit/delete dialogs migrated from the legacy `Modal` to the new `Dialog` (variant `Auto`, width 460) with `Field` + `FormTextInput` + `FormCheckbox` body and `Btn` footer.
3. **Week grid** — CSS Grid: `grid-template-columns: 76px repeat(N, minmax(140px, 1fr))`, `min-width: 920px`. Header row: empty corner cell (`z-index: 3`) + day cells (long weekday name + date + day total in mono, tabular-nums). Body cells: `filled/need` count in mono, `PersonChip`s, **single +/− button** absolutely positioned `top: 6, right: 6`, 20×20. Sticky time column (`position: sticky; left: 0; z-index: 2`). Cell padding `6px 32px 6px 8px` (right padding reserved for the absolute button). State backgrounds: missing-staff `bg-warn-soft`; hover slightly darker; **no per-cell red** (column-wide red optional, decided OUT in master plan).
4. **Working-hours mini overview** — auto-fit grid card per `SalesPerson` with color dot, name, current/target hours, thin progress bar (`warn` if under, `good` if at/over).
5. **Booking log** — restyled on tokens, deleted rows at 50% opacity with a `bad`-tinted `Gelöscht` cell.
6. **Slot edit modal** — switched to `Dialog` + `Field` + `FormSelectInput` + `FormTextInput` (time/integer) + footer `Btn`s.

The interaction model (`Du bearbeitest:` dropdown drives a single +/− button per cell) is **already implemented** in the page coroutine: `current_sales_person` is the editing target, `ShiftPlanAction::AddUserToSlot` and `ShiftPlanAction::RemoveUserFromSlot` are the verbs. The redesign changes only how the cell renders that single +/− affordance — the per-chip × button mentioned in some legacy reference notes does **not** exist today and is **not** added in this change. Chips become display-only.

### Existing atoms and components

- `Btn` / `BtnVariant` (Primary, Secondary, Ghost, Danger) — toolbar action buttons, dialog footers, the cell +/− button host.
- `NavBtn` — square 36 × 36 token-styled icon button used for prev/next-week navigation.
- `PersonChip` — the cell person-pill (color background, dark ink text, no initials).
- `Dialog` / `DialogVariant` — the create/edit/delete shiftplan dialogs and the slot-edit dialog.
- `Field` + `FormTextInput` + `FormSelectInput` + `FormCheckbox` — slot-edit form rows and the `Du bearbeitest:` select.
- `TopBar` — already mounted at the top of the page; not re-rendered by this change.

### Reference HTML

The Schichtplan screen is implemented in plain HTML inside `design_handoff_shifty/Shifty Preview.html`. Use this file for the visual ground truth on toolbar spacing, week-grid column widths, sticky time-column width, cell padding, day-header typography, and progress-bar sizing.

### Data shape constraints

- `state::Shiftplan { week, year, slots: Rc<[Slot]> }` is the loaded structure for week view; `Slot { id, day_of_week, from, to, min_resources, bookings }` carries everything the cell needs. `Booking { sales_person_id, label, background_color, self_added, created, created_by, deleted, deleted_by }` is what each chip displays.
- `WORKING_HOURS_MINI: Rc<[WorkingHoursMini]>` carries `{ sales_person_id, sales_person_name, actual_hours, dynamic_hours, balance_hours }` per row.
- `WEEKLY_SUMMARY_STORE` carries per-day `monday_available_hours … sunday_available_hours` for the day-total in the header row.
- `BOOKING_CONFLICTS_STORE: Rc<[BookingConflict]>` carries the conflict list rendered above the grid.
- `BOOKING_LOG_STORE: Rc<[BookingLog]>` powers the booking-log table.
- `current_sales_person: Signal<Option<SalesPerson>>` is the page-level editing target. The rendered cell needs to know whether `current_sales_person.id` is contained in `slot.bookings`; if yes the cell shows `−` (`bad`-tinted), otherwise `+` (neutral).

## Goals / Non-Goals

**Goals:**
- Rewrite the toolbar in `src/page/shiftplan.rs` so all four visual groups (week navigation, view toggle, action buttons, sales-person select) use design-token classes and the new atoms (`NavBtn`, `Btn`, `Field`, `FormSelectInput`). Remove every `border-black`, `bg-blue-500`, `bg-gray-200`, `bg-slate-200`, `text-blue-600`, `text-orange-600`, `text-gray-500`, `bg-white`, `bg-gray-50` substring from the non-test source of `src/page/shiftplan.rs`.
- Refactor `src/component/shiftplan_tab_bar.rs` to use flat underline tabs with `border-b-2 border-accent text-accent` (active) / `border-b-2 border-transparent text-ink-soft hover:text-ink hover:border-border-strong` (inactive). Move create/edit/delete dialogs from legacy `Modal` to `Dialog` with `Field` + `FormTextInput` + `FormCheckbox` body and `Btn` footer.
- Refactor `src/component/week_view.rs` to render the week grid via CSS Grid (`grid-template-columns: 76px repeat(N, minmax(140px, 1fr))`, `min-width: 920px`). Cells become `position: relative` boxes with the `+`/`−` button absolutely positioned `top: 6px; right: 6px; width: 20px; height: 20px;`. Sticky time column uses `position: sticky; left: 0; z-index: 2`. Header corner cell uses `z-index: 3`. Cell padding is `6px 32px 6px 8px`. The single `+`/`−` button branches on whether `current_sales_person.id` is in `slot.bookings`; show `+` (neutral, `Btn` Ghost-equivalent token classes) when the editing person is **not** in the cell, show `−` (`bad`-tinted `Btn` Danger-equivalent classes) when they **are**.
- Keep the `WeekViewButtonTypes::Dropdown` and `WeekViewButtonTypes::None` branches working as today (structure mode + read-only). Dropdown-mode cells still use the `…` dropdown trigger (also restyled on tokens) instead of the +/− button.
- Refactor `src/component/working_hours_mini_overview.rs` into an auto-fit grid (`grid-template-columns: repeat(auto-fit, minmax(220px, 1fr))`) of cards. Each card carries `bg-surface border border-border rounded-md p-2`, a row with the sales-person color dot + name, a row with `actual_hours / dynamic_hours` in mono with tabular-nums, and a thin progress bar (`warn` color when `actual_hours < dynamic_hours`, `good` color when `actual_hours >= dynamic_hours`). Selected sales-person card carries an `accent`-tinted highlight (`bg-accent-soft` background + `border-accent`).
- Refactor `src/component/booking_log_table.rs` to use design tokens (`bg-surface`, `bg-surface-alt`, `border-border`, `text-ink`, `text-ink-muted`, `text-ink-soft`). Deleted rows render with `opacity-50` and a `bad`-tinted `Gelöscht` cell (replacing today's `line-through text-gray-500`). All filter inputs use the form-input token classes; the Clear-Filters button uses `Btn` Secondary; deleted-only / active-only / all dropdowns keep their values.
- Refactor `src/component/slot_edit.rs` to use `Dialog` (variant `Auto`, width 460) instead of legacy `Modal`. The body uses `Field` rows wrapping `FormSelectInput` (weekday), token-styled time inputs (from / to), and a token-styled integer input (min persons). The footer uses `Btn` Secondary Cancel + `Btn` Primary Save.
- Add new i18n keys: `Key::ShiftplanFilledOfNeed` (the `filled/need` count cell, format string `{filled}/{need}`), `Key::ShiftplanWeekTotal` (header day-total label format), `Key::ShiftplanLastWeek` ("Letzte Woche" copy button), `Key::ShiftplanCellAddTitle` (aria-label for the `+` button: "Add me to this slot"), `Key::ShiftplanCellRemoveTitle` (aria-label for the `−` button: "Remove me from this slot"), `Key::BookingLogDeletedTag` ("Gelöscht" / "Deleted" / "Smazáno" — the deleted-cell badge text). Reuse existing `ViewModeWeek`, `ViewModeDay`, `ShiftplanCalendarWeek`, `ShiftplanEditAs`, `ShiftplanYouAre`, `Save`, `Cancel`, `BookingLogDeleted`, `BookingLogTitle`, `BookingLogShow`, `BookingLogHide`.
- All non-test source of `src/page/shiftplan.rs`, `src/component/week_view.rs`, `src/component/shiftplan_tab_bar.rs`, `src/component/working_hours_mini_overview.rs`, `src/component/booking_log_table.rs`, and `src/component/slot_edit.rs` SHALL NOT contain any of `bg-gray-`, `bg-white`, `text-gray-`, `text-blue-`, `text-red-`, `text-green-`, `text-orange-`, `bg-blue-`, `bg-green-`, `bg-red-`, `bg-slate-`, `border-gray-`, `border-black` after the rewrite.
- Print stylesheet integrity: the `.print:hidden` toggles SHALL still hide the booking-log filter row, the dropdown trigger, the zoom selector, the calendar-export links, and the report panel. The week grid SHALL still print the slot grid; QA pass on actual print preview required (Tasks §10).

**Non-Goals:**
- No per-cell red conflict tinting. Conflict surfacing stays as today: a `<ul>` above the grid that lists conflicts and clicking a row swaps the editing sales person. The master plan explicitly excludes per-cell red.
- No tag (single-day) view logic changes. The existing `DayView` / `DayAggregateView` keep their current internal structure; only token-class sweep applies.
- No backend changes. No new endpoints, no new fields on existing endpoints, no migrations. The redesign is pure frontend.
- No removal of legacy `Modal`, `Button`, `TextInput`, `Form`, `FormGroup`, `FormPair`, `Select`, `TimeInput`, `IntegerInput` from `base_components.rs`. Cleanup change `99` removes them once every consumer migrates.
- No restyling of `src/component/day_aggregate_view.rs` beyond a token-class sweep on the visible button bar (the structural day-aggregate layout stays as-is). Day-aggregate-specific redesign is out of scope here.
- No URL persistence of the view-mode toggle, the selected shiftplan tab, or the change-structure-mode flag. Today these are page-local signals and stay that way.
- No new permission gates; existing `is_shiftplanner`, `is_shift_editor`, `is_hr` checks keep their current branches.
- No drag-and-drop, inline rename, or keyboard navigation across cells. Cell `+/−` is mouse-only / touch-only as today.
- No initials inside the color dots in the working-hours mini overview. The dot is decorative; the name is in plain text next to it.
- No change to the `change_structure_mode` semantics — when active, cells render the dropdown menu (`WeekViewButtonTypes::Dropdown`), not the `+/−` button. The dropdown menu items (`Log slot id`, `Edit slot`, `Remove slot`) keep their existing handlers.

## Decisions

### 1. Week grid layout — CSS Grid replaces flex-of-DayView

Three layout options were considered:

| Option | Pros | Cons |
|---|---|---|
| Keep `flex flex-row` of seven `DayView` columns + sticky `TimeView` (today) | Smallest diff; existing component split is preserved | Cell-level styling tightly coupled to absolute-positioned `ColumnViewSlot`; sticky time column needs separate flex slot; column widths drift by content; `min-width` enforcement needs JS or rigid `min-w-*` per child |
| CSS Grid: `grid-template-columns: 76px repeat(N, minmax(140px, 1fr))` on a single grid container, with the time column as the first grid column and each weekday as one column. Cells use `grid-row: span <duration>` for time placement, OR each weekday is itself a sub-grid of `grid-template-rows: repeat(<H * 4>, 14px)` | Single source of truth for column widths and `min-width: 920px`; sticky time column is column 1 of the same grid; header row aligns with body via grid | Larger diff; existing `DayView` and `ColumnView` components no longer match the layout; absolute-positioned `ColumnViewSlot` math has to move to grid placement |
| CSS Grid for the column scaffold + absolute-positioned `ColumnViewSlot` inside each weekday column (hybrid) | Keeps existing slot-positioning math (`top: y * SCALING + offset; height: ...`) intact; only the wrapper changes | Two coordinate systems (grid columns × absolute pixels) coexist; cells overlap the column boundary cleanly because each column is its own positioning context |

**Chosen: hybrid (option 3).** Reasons: (a) the existing `ColumnViewSlot` positioning math is correct and well-tested — moving every slot to grid-row spans would be a huge ripple for purely visual gain; (b) the user-visible improvement (column widths, sticky time column, single `+/−` button) is achievable without rewriting slot-time math; (c) the `print:` stylesheet works either way, but the hybrid keeps print rendering identical to today's layout.

Implementation:

```rust
div {
    class: "grid",
    style: format!(
        "grid-template-columns: 76px repeat({}, minmax(140px, 1fr)); min-width: 920px;",
        if has_sunday { 7 } else { 6 }
    ),
    // Header row: corner cell (sticky) + day cells
    div {
        class: "sticky top-0 left-0 bg-surface border-b border-border",
        style: "z-index: 3;",
        // empty corner
    }
    for weekday in visible_weekdays {
        div {
            class: "sticky top-0 bg-surface border-b border-border px-2 py-1",
            style: "z-index: 1;",
            div { class: "text-sm font-semibold text-ink", "{weekday_long_name}" }
            div { class: "text-xs text-ink-muted", "{date_str}" }
            div { class: "text-xs font-mono tabular-nums text-ink-muted", "{day_total}" }
        }
    }
    // Body row: sticky time column + day columns
    div {
        class: "sticky left-0 bg-surface border-r border-border",
        style: "z-index: 2;",
        TimeView { start, end }
    }
    for weekday in visible_weekdays {
        div {
            class: "relative border-r border-border",
            style: "height: {grid_height}px;",
            for slot in slots_for_weekday {
                CellSlot { /* absolute positioning unchanged */ }
            }
        }
    }
}
```

The `min-width: 920px` is enforced by the inline style on the grid container; the parent (`overflow-x-auto`) handles horizontal scroll on narrow viewports.

### 2. Single +/− button replaces three-button cell stack

Today's cell stack renders three buttons in a vertical column at the right edge: `+`, `−`, and `…` (dropdown). Visibility is gated by `WeekViewButtonTypes::AddRemove` / `WeekViewButtonTypes::Dropdown`. The redesign collapses the `+`/`−` pair to a single absolutely positioned 20×20 button:

- **Editing person not in cell** → render `+` (neutral, `Btn` Ghost-equivalent token classes: `bg-surface-alt text-ink hover:bg-surface-soft border border-border-strong`).
- **Editing person in cell** → render `−` (`bad`-tinted: `bg-bad-soft text-bad hover:bg-bad-soft/80 border border-bad`).
- **No editing person selected** → render no button at all (cell is read-only that frame).

The button is positioned `position: absolute; top: 6px; right: 6px; width: 20px; height: 20px;` with `z-index: 1` so it overlays the chip-flow content. The cell's right padding of `32px` reserves space so chips never visually collide with the button.

The `…` dropdown button (used in `WeekViewButtonTypes::Dropdown` when `change_structure_mode = true`) renders in the same absolute slot, replacing the `+`/`−` rather than coexisting. It uses the same 20×20 footprint with `Btn` Ghost-equivalent classes.

The `WeekViewButtonTypes::None` branch (older-than-2-weeks read-only for non-HR) renders no button at all — the cell shows chips only.

This branching lives inside the cell render (`ColumnViewSlot` or its replacement). The signature gains an `editing_person_in_cell: bool` field so the caller can resolve the membership check once per slot. The `add_event` / `remove_event` handlers on the existing `WeekViewProps` are preserved; they continue to receive the `Slot` via `EventHandler<Slot>`.

**Alternative considered**: keep both `+` and `−` buttons, but only show one based on the membership check. Rejected — same end-user behavior, larger cell footprint. The single-button design wins on real estate.

**Alternative considered**: keep the per-chip `×` remove button. Rejected — it doesn't exist in today's code (chips are display-only with no remove control), and the proposal explicitly says `Per-chip × buttons removed (chips become display-only)` (which describes the target state, not a removal). No code path needs to be deleted; chips already render as display-only `<p>` elements.

### 3. PersonChip integration in cells

Cells today render chips as bare `<p>` elements with inline `background-color`. The redesign replaces them with the existing `PersonChip` atom (`src/component/atoms/person_chip.rs`):

```rust
PersonChip {
    name: booking.label.clone(),
    color: Some(booking.background_color.clone()),
    is_self: Some(booking.self_added),
}
```

Reasons: (a) `PersonChip` already enforces the redesign rules (dark ink text, no initials, pastel background); (b) it carries the `*` self-added marker via an `is_self` prop (or equivalent — see the atom's actual prop list); (c) keeping the chip as an atom makes the SSR test trivial.

Fallback when `PersonChip` lacks a tooltip slot: keep the existing `onmousedown` / `onmouseup` / `ontouchstart` tooltip wiring on a wrapping `<div>` around `PersonChip`. The tooltip-service interaction (`TooltipAction::ShowTooltip`) does not change.

The min-resources `filled/need` chip stays as a dedicated cell-level render (not a `PersonChip`) using `font-mono tabular-nums text-ink` classes; understaffed renders with `bg-warn-soft` and `text-warn`, fully-staffed renders with no background tint. This is a token-class swap from `#ffcccc` / `#fff` inline styles.

### 4. Sticky time column — sub-grid vs. separate flex slot

Today the time column is a separate flex child outside the day-columns wrapper. The redesign moves it into the same CSS Grid as the day columns (column 1 of `grid-template-columns: 76px repeat(N, minmax(140px, 1fr))`).

**Sticky positioning details**:
- Time column: `position: sticky; left: 0; z-index: 2; background: var(--surface);` so the column floats over weekday columns during horizontal scroll.
- Header row: `position: sticky; top: 0; z-index: 1` for day-header cells; `z-index: 3` for the corner (intersection of sticky-top and sticky-left).

This change requires the `TimeView` component to render hour labels in a way that aligns with the absolute-positioning math used inside each weekday column. The `SCALING = 75.0` constant stays the same, so the time labels still map 1 hour → 75 px. The time column container is `position: relative; height: <grid_height>px;` and `TimeView` renders its labels as absolute children — same pattern as today.

### 5. Day header — long name + date + day total

Today's day header is `<weekday>, <date> | <header>` concatenated into a single `<p>` inside the `ColumnView` title slot. The redesign expands it to three stacked rows:

- Long weekday name (`text-sm font-semibold text-ink`).
- Date (`text-xs text-ink-muted`), formatted via `i18n.format_date`.
- Day total in mono (`text-xs font-mono tabular-nums text-ink-muted`), formatted as `{:.1}h` from the existing `weekly_summary` data.

The `header` prop on `DayView` (which today passes the day-total into the title) is replaced by separate signals on the new grid container. The day total is read directly from `WEEKLY_SUMMARY_STORE` inside the week-grid component rather than passed in as a prop. This removes the `weekday_headers: Vec<(Weekday, Rc<str>)>` prop from `WeekViewProps` (deviation from "no logic change" — see risks).

**Sub-decision**: keep `weekday_headers` as a deprecated prop for one release cycle, log a `tracing::warn` when it's set, and ignore it in the rendered output. Rejected — no other consumer of `WeekView` exists; the `shiftplan.rs` page is the only call site (verified by `grep WeekView src --include='*.rs'`). Remove the prop directly.

### 6. View toggle — segmented control inside `bg-surface-alt` pill

Today's view toggle is two `<button>` elements with hard-coded `bg-blue-500 text-white` (active) and `bg-gray-200 hover:bg-gray-300` (inactive). The redesign uses a segmented control inside a `bg-surface-alt rounded-md p-1` pill:

```rust
div { class: "inline-flex bg-surface-alt rounded-md p-1 gap-0.5",
    button {
        class: if *view_mode.read() == ViewMode::Week {
            "px-3 py-1 text-sm font-medium rounded bg-surface text-ink shadow-sm"
        } else {
            "px-3 py-1 text-sm font-medium text-ink-soft hover:text-ink"
        },
        onclick: move |_| view_mode.set(ViewMode::Week),
        {i18n.t(Key::ViewModeWeek)}
    }
    button {
        class: ...,  // mirror for Day
        onclick: move |_| { view_mode.set(ViewMode::Day); cr.send(ShiftPlanAction::LoadDayAggregate); },
        {i18n.t(Key::ViewModeDay)}
    }
}
```

Both buttons render at `text-sm`. The active button has a raised look via `bg-surface shadow-sm`; the inactive button is flat against the pill background.

### 7. Week-navigation — `NavBtn` + week label

Today: two `<button>` elements with hard-coded borders sandwich a `<div>{calendar_week_str}</div>`. The redesign wraps prev/next in `NavBtn` (the new 36 × 36 token-styled icon button) and keeps the week label between them as a `<span class="text-base font-medium text-ink">`. The `calendar_week_str` produced by `i18n.t_m(Key::ShiftplanCalendarWeek, ...)` already carries the `KW <n> · <year> · <date>` format; no new key required.

### 8. `Du bearbeitest:` — `Field` + `FormSelectInput`

Today: `"Edit as" + <select class="bg-slate-200 p-1 rounded-md ml-2">` with raw `<option>` children. The redesign uses the new `Field` + `FormSelectInput`:

```rust
Field {
    label: i18n.t(Key::ShiftplanEditAs),
    input: rsx! {
        FormSelectInput {
            value: current_sales_person.read().as_ref().map(|sp| sp.id.to_string()),
            options: sales_persons.iter().filter(|sp| !sp.inactive).map(|sp| (sp.id.to_string(), sp.name.clone())).collect(),
            on_change: move |value| { /* parse Uuid, dispatch UpdateSalesPerson */ },
        }
    },
}
```

The `Field` label sits above the select (matching the form-row pattern used in change 04). The select itself uses the form-input tokens from `04`. The non-shiftplanner branch (`is_shiftplanner = false`) keeps the read-only `"Du bist: <chip>"` rendering, but the chip becomes a `PersonChip` instead of a `<span class="bg-slate-200">`.

### 9. Working-hours mini overview — auto-fit grid of cards

Today: a vertical list `flex flex-col max-w-96`. The redesign uses an auto-fit grid:

```rust
div {
    class: "grid gap-2",
    style: "grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));",
    for working_hour in working_hours.iter() {
        div {
            class: format!("bg-surface border rounded-md p-2 cursor-pointer {}",
                if Some(working_hour.sales_person_id) == selected_sales_person_id { "bg-accent-soft border-accent" } else { "border-border" }
            ),
            // header row: dot + name
            div { class: "flex items-center gap-2 mb-1",
                span { class: "w-2.5 h-2.5 rounded-full", style: "background-color: {working_hour.background_color}" }
                span { class: "text-sm font-medium text-ink truncate", "{working_hour.sales_person_name}" }
            }
            // hours row: actual / target (mono)
            div { class: "flex items-baseline gap-1 text-xs font-mono tabular-nums text-ink-muted",
                span { "{actual_hours}" }
                span { "/" }
                span { "{dynamic_hours}" }
                if show_balance {
                    span { class: "ml-auto", "({balance_hours})" }
                }
            }
            // progress bar
            div {
                class: "h-1 bg-surface-alt rounded-full mt-2 overflow-hidden",
                div {
                    class: format!("h-full {}", if working_hour.actual_hours < working_hour.dynamic_hours { "bg-warn" } else { "bg-good" }),
                    style: format!("width: {}%;", percent_of_target(working_hour.actual_hours, working_hour.dynamic_hours)),
                }
            }
        }
    }
}
```

`WorkingHoursMini` carries `sales_person_id`, `sales_person_name`, `actual_hours`, `dynamic_hours`, `balance_hours`. **It does not carry a `background_color` field today** — see decision 10.

### 10. Color-dot data flow for the mini overview

Today's mini overview has no color dot. The redesign needs a per-row color dot. The `WorkingHoursMini` struct does not carry `background_color`. Two options:

| Option | Trade-off |
|---|---|
| Add `background_color: ImStr` to `WorkingHoursMini` and populate it inside the `WorkingHoursMiniAction::LoadWorkingHoursMini` handler | Pure additive; one place to populate; matches the data-flow pattern used elsewhere |
| Look up `background_color` at render time by joining against `sales_persons_resource` in `shiftplan.rs` | Avoids state-shape change; adds a render-time fallback when the join misses |

**Chosen: option 1 — add the field.** Reasons: (a) the data is cheap to populate at load time (the loader already returns sales-person tuples or can be extended with one extra `SalesPerson` lookup per row); (b) render-time joins introduce a load-order race where the mini overview renders briefly without dots until the sales-person resource resolves; (c) the loader already runs once per week change, so the additional field doesn't multiply request counts.

The migration for `WorkingHoursMini` is field-additive only — existing consumers just don't read it. The loader change is in `loader::load_working_hours_mini` (or wherever the action handler lives); the field is populated from the same SalesPerson list the action already enumerates.

If the loader does not have access to the `SalesPerson` color (e.g., a backend response without color), the field defaults to `ImStr::from("#cccccc")` (a neutral gray) and the dot still renders. Documented under risks.

### 11. Booking log — opacity-50 + bad-tinted Gelöscht cell

Today: deleted rows render with `line-through text-gray-500` on the entire `<tr>`. The redesign:

- Deleted rows: `opacity-50` on the `<tr>`. Removes the line-through (visual line-through over fine-print mono text reduces legibility; opacity is the canonical "deleted but visible" cue).
- The `Gelöscht` cell (today: empty when not deleted, datetime when deleted) gets a small badge `<span class="inline-flex px-2 py-0.5 rounded-sm text-xs font-medium bg-bad-soft text-bad">Gelöscht</span>` followed by the datetime in `text-xs text-ink-muted`. Active rows render no badge in this cell.
- The header row uses `bg-surface-alt`. The body rows hover to `bg-surface-alt`. The filter section uses `bg-surface border border-border` instead of `bg-gray-50 border border-gray-200`.

The "Gelöscht" badge text is a new i18n key `Key::BookingLogDeletedTag`. The existing `Key::BookingLogDeleted` is the column header text and stays.

### 12. Slot-edit dialog migration

`src/component/slot_edit.rs` today renders inside the legacy `Modal` and uses legacy `Form`/`FormGroup`/`FormPair`/`Select`/`TimeInput`/`IntegerInput`/`Button`. The redesign:

- Replace `Modal` with `Dialog` (variant `Auto`, width 460). The `props.visible` flag drives `Dialog::open`. The `props.on_cancel` callback drives `Dialog::on_close` (so backdrop click and ESC dispatch Cancel).
- Replace `Form` + `FormGroup`/`FormPair` with `Field` rows. Each `Field` carries the label and wraps the input.
- Replace legacy `Select` with `FormSelectInput`. Replace legacy `TimeInput` with a token-styled time input (likely a thin wrapper over an `<input type="time">` using the form-input token classes; the existing `TimeInput` in `base_components.rs` is migrated, not deleted, in `99`).
- Replace legacy `IntegerInput` with a token-styled integer input. Same migration path.
- Replace the body `<h1>` title with the `Dialog::title` prop (passed as `ImStr`).
- Replace footer Save/Cancel buttons with `Btn` Primary Save and `Btn` Secondary Cancel inside `Dialog::footer`.
- The error message (`props.has_errors`) renders as a `text-bad` paragraph above the footer. No layout change beyond token swap.

### 13. Tab-bar dialog migration (shiftplan create/edit/delete)

Today: `src/component/shiftplan_tab_bar.rs` mounts two legacy `Modal`s for create/edit and delete-confirm. The redesign:

- **Create/edit dialog**: `Dialog` variant `Auto`, width 460, title `i18n.t(Key::ShiftplanCreateTitle)` (new key) or `Key::ShiftplanEditTitle` (new key). Body: `Field` with `FormTextInput` (name) + `Field` with `FormCheckbox` (is_planning). Footer: `Btn` Secondary Cancel + `Btn` Primary Confirm (label `Erstellen` / `Speichern` via existing `Key::Save` / new `Key::Create`).
- **Delete-confirm dialog**: `Dialog` variant `Auto`, width 420, title `Key::ShiftplanDeleteConfirmTitle` (new key). Body: a paragraph with `Key::ShiftplanDeleteConfirmBody`. Footer: `Btn` Secondary Cancel + `Btn` Danger Delete.

The hard-coded German strings inside the legacy dialogs (`"Neuen Shiftplan erstellen"`, `"Shiftplan löschen?"`, etc.) become real i18n keys. New keys: `ShiftplanCreateTitle`, `ShiftplanEditTitle`, `ShiftplanDeleteConfirmTitle`, `ShiftplanDeleteConfirmBody`, `ShiftplanIsPlanningLabel`, `Create`. Translations in en/de/cs.

### 14. New i18n keys

Eleven new keys cover the redesign:

- `Key::ShiftplanFilledOfNeed` — format string for the cell `filled/need` count (e.g., `"{filled}/{need}"`). Parameter substitution via `t_m`.
- `Key::ShiftplanLastWeek` — "Letzte Woche" / "Last week" / "Minulý týden" — copy-from-previous-week button label.
- `Key::ShiftplanCellAddTitle` — "Add me" / aria-label for the `+` button.
- `Key::ShiftplanCellRemoveTitle` — "Remove me" / aria-label for the `−` button.
- `Key::ShiftplanCreateTitle` — "Create shiftplan" / "Neuen Shiftplan erstellen" / Czech equivalent.
- `Key::ShiftplanEditTitle` — "Edit shiftplan" / "Shiftplan bearbeiten" / Czech equivalent.
- `Key::ShiftplanDeleteConfirmTitle` — "Delete shiftplan" / "Shiftplan löschen" / Czech equivalent.
- `Key::ShiftplanDeleteConfirmBody` — "Are you sure you want to delete shiftplan {name}? This cannot be undone." / German + Czech.
- `Key::ShiftplanIsPlanningLabel` — "Planning only" / "Nur Planung" / Czech equivalent.
- `Key::Create` — "Create" / "Erstellen" / "Vytvořit" — generic create button label, reused.
- `Key::BookingLogDeletedTag` — "Deleted" / "Gelöscht" / "Smazáno" — the deleted-row badge text.

`ShiftplanWeekTotal` is **not** added — the day-total is rendered as plain `{:.1}h` (no localizer-visible label needed; the column header is the weekday name and the date).

### 15. File-level plan

**Modified files:**
- `src/page/shiftplan.rs` — toolbar rewrite, view-toggle rewrite, sales-person select rewrite, conflict list token sweep, week-message editor token sweep, shiftplan-report panel token sweep, booking-log panel token sweep. Estimated ~300 LOC change.
- `src/component/shiftplan_tab_bar.rs` — flat-underline tabs, `Dialog`-based create/edit/delete. Estimated full rewrite (~200 LOC).
- `src/component/week_view.rs` — CSS Grid scaffold, sticky time column inside grid, single +/− button, header rewrite. Estimated ~250 LOC change.
- `src/component/working_hours_mini_overview.rs` — auto-fit card grid, color dot, progress bar. Estimated full rewrite (~80 LOC).
- `src/component/booking_log_table.rs` — token sweep, opacity-50 deleted rows, `Gelöscht` badge. Estimated ~80 LOC change.
- `src/component/slot_edit.rs` — `Dialog`-based, `Field` + form-atoms body. Estimated full rewrite (~150 LOC).
- `src/state/employee_work_details.rs` (or wherever `WorkingHoursMini` lives) — add `background_color: ImStr` field.
- `src/loader.rs` (or `src/service/working_hours_mini.rs`) — populate `background_color` when loading `WorkingHoursMini`.
- `src/i18n/mod.rs`, `src/i18n/en.rs`, `src/i18n/de.rs`, `src/i18n/cs.rs` — eleven new keys.

**No new files.** All visual changes happen in existing files.

Estimated diff: ~1100 LOC change, ~500 LOC net addition.

### 16. Test strategy

Three tiers, mirroring change 07/08:

**Unit tests (pure functions):**
- `cell_button_branching_when_editing_person_present` — function returns `Remove` variant when editing person id is in the booking list.
- `cell_button_branching_when_editing_person_absent` — function returns `Add` variant when editing person id is not in the booking list.
- `cell_button_branching_when_no_editing_person` — function returns `None` variant when `current_sales_person` is `None`.
- `progress_bar_color_under_target` — helper returns `bg-warn` when actual < target.
- `progress_bar_color_at_or_over_target` — helper returns `bg-good` when actual >= target.
- `progress_bar_percent_caps_at_100` — helper clamps to 100% when actual exceeds target.
- `min_resources_classes_understaffed` — returns `bg-warn-soft text-warn` when `bookings.len() < min_resources`.
- `min_resources_classes_fully_staffed` — returns no soft tint when fully staffed.
- `i18n_redesign_keys_present_in_all_locales` — all eleven new keys return non-empty strings in En/De/Cs.

**SSR tests (rendered HTML assertions):**
- Toolbar: prev/next buttons render as `NavBtn` (assert via `data-testid` or class fingerprint); week label renders the `KW <n> · ...` string; `Letzte Woche` button is a `Btn` Secondary; the `Du bearbeitest:` select is a `Field` + `FormSelectInput`.
- View toggle: the active button carries `bg-surface text-ink shadow-sm`; the inactive one carries `text-ink-soft`. Both share `text-sm font-medium`.
- Tab bar: active tab carries `border-accent text-accent`; inactive carries `border-transparent text-ink-soft`. Create/edit dialog renders a `Dialog` with a `Field` + `FormTextInput` + `Field` + `FormCheckbox`. Delete dialog renders a `Dialog` with a Danger button in the footer.
- Week grid: rendered HTML contains a `style` attribute with `grid-template-columns: 76px repeat(N, minmax(140px, 1fr))` and `min-width: 920px`. Header row contains corner cell with `z-index: 3`. Time column has `position: sticky` and `left: 0`. Each weekday header carries weekday name + date + day total in mono.
- Cell button: a cell whose slot bookings include the editing person renders a button with `bg-bad-soft` and `−` glyph; a cell without renders a button with `bg-surface-alt` and `+` glyph; a cell rendered with `current_sales_person = None` renders no button.
- PersonChip in cells: each chip carries `class="person-pill"` (or whatever the atom emits); the chip text is the booking label; no `<initials>` or 2-letter abbreviation appears.
- Min-resources: understaffed cell renders `<span>{filled}/{need}</span>` with `bg-warn-soft text-warn`; fully-staffed renders no tint.
- Working-hours mini overview: renders an auto-fit grid; each card carries the color dot inline-style; progress bar carries `bg-warn` when under, `bg-good` when at/over; selected card carries `bg-accent-soft border-accent`.
- Booking-log table: deleted row carries `opacity-50`; the deleted-cell carries the `bg-bad-soft text-bad` Gelöscht badge; active rows carry no badge; filter section carries `bg-surface border border-border` (not `bg-gray-50`).
- Slot-edit dialog: renders inside `Dialog` (assert by `role="dialog"` plus the `Dialog`-specific style fingerprints from `Dialog`'s tests); body contains `Field` + `FormSelectInput` for the weekday; footer contains Cancel + Save `Btn`s.
- Token sweep: the non-test source of each redesigned file SHALL NOT contain any of `bg-gray-`, `bg-white`, `text-gray-`, `text-blue-`, `text-red-`, `text-green-`, `text-orange-`, `bg-blue-`, `bg-green-`, `bg-red-`, `bg-slate-`, `border-gray-`, `border-black`.

Tests live next to their source under `#[cfg(test)] mod tests`.

## Risks / Trade-offs

**[Largest visual diff in the project]** → The week grid is the most-used screen and the rewrite touches its rendering at every level (layout, cell, header, sticky, button). Mitigation: ship with the existing data-flow (signals, actions, loader paths) intact so functional regressions are limited to rendering. Heavy SSR test coverage on cell-button branching and grid layout. Manual QA pass on real data (week with 3+ shiftplans, week with conflicts, week with empty days) before merge.

**[CSS Grid + absolute positioning hybrid carries two coordinate systems]** → Time-of-day positioning (today's `top: y * SCALING`) coexists with grid column placement. Cells could mis-align if `SCALING` changes or if grid column widths drift. Mitigation: keep `SCALING` constant; assert column-width style in SSR tests; manual QA on horizontal scroll and zoom.

**[Sticky time column over CSS Grid is fragile in some browsers]** → `position: sticky` inside a CSS Grid container has known edge cases in Safari (column 1 sticky-left + column 2+ scrolling). Mitigation: existing reference implementation in `Shifty Preview.html` is known-good; ship the same pattern (with `bg-surface` so the sticky column visually masks the columns underneath); add manual QA on Safari/iOS.

**[Removing `weekday_headers` prop from `WeekViewProps`]** → A behavior-touching change, even though no other consumer exists. Mitigation: documented under decision 5. Verified by `grep WeekView` in the page source — only `shiftplan.rs` calls it. The day-total is now read inline from `WEEKLY_SUMMARY_STORE` instead of passed in.

**[`WorkingHoursMini` field addition is data-shape change]** → Adding `background_color: ImStr` to `WorkingHoursMini` is a non-breaking field addition, but every existing serializer/deserializer (if any) and every existing test that constructs a `WorkingHoursMini` literal needs the new field. Mitigation: add a `Default` impl; default literal constructions use `..Default::default()` where possible; otherwise add the field with a sensible default in each test fixture.

**[Print stylesheet drift]** → The redesigned toolbar, tab bar, mini overview, and booking-log all carry `print:hidden` toggles today. The redesign moves them to design-token classes; `print:hidden` still works. Risk is that the new CSS Grid week grid changes its print rendering (page breaks across grid rows, column widths). Mitigation: explicit print-preview QA pass (Tasks §10) covering a one-week and a multi-week print. Acceptance: the existing print layout (no toolbar, no mini overview, just the grid) is preserved or improved.

**[New i18n keys must be translated to all three locales]** → Eleven new keys × three locales = thirty-three new translations. Risk: a typo or missing translation lands in production. Mitigation: a unit test (`i18n_redesign_keys_present_in_all_locales`) asserts every new key returns a non-empty string in En/De/Cs.

**[`Btn` Ghost / Danger variant tokens used inline for the cell button]** → The single `+/−` button is small (20×20) and does not use the full `Btn` atom (which carries padding suited for toolbar use). Cell button uses the same token classes as `Btn` Ghost / Danger but renders inline as a `<button>`. Risk: cell-button styling drifts from `Btn` over time. Mitigation: extract a small `CellSlotButton` helper that renders the `+/−` glyph with the appropriate token classes, kept next to `WeekView` so renames stay coordinated.

**[Slot-edit migration introduces a Dialog within the page render tree]** → Today the slot-edit modal mounts unconditionally and toggles via `props.visible`. The new `Dialog` early-returns on `open: false`. The render-tree shape is the same (no DOM element when closed), but ESC and backdrop-click behavior changes (today's `Modal` doesn't have ESC; the new `Dialog` does). Mitigation: the `on_cancel` handler stays unchanged; ESC and backdrop now also dispatch `SlotEditAction::Cancel` for free.

**[Tooltip wiring on chips relies on `onmousedown` / `ontouchstart` timers]** → The existing chip rendering carries 500 ms-delay tooltip logic. Wrapping the chip in a `PersonChip` atom may move the event handlers up one layer in the DOM tree. Mitigation: keep the wrapping `<div>` that holds the event handlers; render `PersonChip` as the visual surface inside it. Tooltip behavior unchanged.

**[Booking log "Deleted" cell badge changes the cell content shape]** → Today the deleted cell is empty when not deleted; tests that check for empty-string content may need updating. Mitigation: only one consumer (page-level); the test updates are bounded.

**[`WeekViewButtonTypes::Dropdown` mode changes from three-button stack to single dropdown trigger]** → The `…` dropdown trigger replaces the cell's `+`/`−` slot when `change_structure_mode = true`. Click-targets are smaller. Mitigation: 20×20 is still a comfortable touch target (40×40 is recommended, but the cell's right padding gives the button a 32×32 effective hit area).

**[Day-aggregate view (`ViewMode::Day`) only gets a token-class sweep, not a layout rewrite]** → The day view will look slightly inconsistent with the week view (token-styled controls inside an old layout). Mitigation: documented; a future redesign change can extend the day view if desired. The toggle still works.
