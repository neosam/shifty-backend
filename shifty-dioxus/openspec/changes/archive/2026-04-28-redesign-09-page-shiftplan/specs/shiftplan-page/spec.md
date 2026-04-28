## ADDED Requirements

### Requirement: Redesigned toolbar with tokenized controls
The shiftplan page SHALL render a top toolbar containing four logical groups, all using the new design-token classes and atoms: (1) prev/next-week navigation using two `NavBtn` instances with the calendar-week label `KW <n> · <date range>` (produced by `i18n.t_m(Key::ShiftplanCalendarWeek, ...)`) between them, (2) a Woche/Tag segmented view toggle inside a `bg-surface-alt rounded-md p-1` pill where the active option carries `bg-surface text-ink shadow-sm` and the inactive one carries `text-ink-soft hover:text-ink`, (3) action controls including a `Btn` Secondary labeled `i18n.t(Key::ShiftplanLastWeek)` that dispatches `ShiftPlanAction::CopyFromPreviousWeek` and the existing iCal export links restyled with `text-accent` (4) a `Field` + `FormSelectInput` for `Du bearbeitest:` whose label equals `i18n.t(Key::ShiftplanEditAs)` and whose options list every active sales person. The non-shiftplanner branch SHALL render `i18n.t(Key::ShiftplanYouAre)` followed by a `PersonChip` carrying the current sales person's name and color (no `<span class="bg-slate-200">` rendering).

#### Scenario: Prev/next-week buttons render as NavBtn
- **WHEN** the toolbar is rendered for any week
- **THEN** the prev and next controls SHALL each be a `NavBtn` (assert by class fingerprint matching the `NavBtn` atom output) and the week label between them SHALL contain the substring produced by `i18n.t_m(Key::ShiftplanCalendarWeek, [...])`

#### Scenario: View toggle uses tokenized segmented control
- **WHEN** view mode is `Week`
- **THEN** the Woche button's class list SHALL include `bg-surface`, `text-ink`, and `shadow-sm`, AND the Tag button's class list SHALL include `text-ink-soft`

#### Scenario: View toggle swaps active styling
- **WHEN** view mode is `Day`
- **THEN** the Tag button's class list SHALL include `bg-surface`, `text-ink`, and `shadow-sm`, AND the Woche button's class list SHALL include `text-ink-soft`

#### Scenario: Letzte Woche button is a Btn Secondary
- **WHEN** the user is a shift editor and the toolbar is rendered
- **THEN** the rendered HTML SHALL contain a `Btn` Secondary whose visible text equals `i18n.t(Key::ShiftplanLastWeek)`

#### Scenario: Du bearbeitest select uses Field and FormSelectInput
- **WHEN** the user is a shiftplanner and the sales-person resource has loaded
- **THEN** the toolbar SHALL render a `Field` whose label is `i18n.t(Key::ShiftplanEditAs)` wrapping a `FormSelectInput` whose option list contains every loaded non-inactive sales person

#### Scenario: Non-shiftplanner sees PersonChip instead of select
- **WHEN** the user is not a shiftplanner and a current sales person is set
- **THEN** the toolbar SHALL render `i18n.t(Key::ShiftplanYouAre)` followed by a `PersonChip` for that sales person AND SHALL NOT render a `<select>` element

### Requirement: Restyled shiftplan tab bar with flat underline tabs
The shiftplan tab bar SHALL render each tab as a `<button>` element whose class list resolves to `border-b-2 border-accent text-accent` when the tab is active and `border-b-2 border-transparent text-ink-soft hover:text-ink hover:border-border-strong` when inactive. The container SHALL carry `border-b border-border` to provide the underline strip beneath unrendered area. In structure mode, the per-tab `✕` delete affordance SHALL render with `text-bad-soft hover:text-bad` (no `text-red-400`), and the `+` create affordance SHALL render as a `Btn` Ghost with token classes (no `text-green-600`).

#### Scenario: Active tab carries accent classes
- **WHEN** a shiftplan tab matches the selected id
- **THEN** the tab `<button>` class list SHALL include `border-b-2`, `border-accent`, and `text-accent`

#### Scenario: Inactive tab carries muted classes
- **WHEN** a shiftplan tab does not match the selected id
- **THEN** the tab `<button>` class list SHALL include `border-b-2`, `border-transparent`, and `text-ink-soft`

#### Scenario: Tab bar renders no legacy color classes
- **WHEN** the non-test source of `src/component/shiftplan_tab_bar.rs` is inspected
- **THEN** it SHALL NOT contain any of `bg-gray-`, `text-gray-`, `text-blue-`, `text-red-`, `text-green-`, `bg-blue-`, `bg-red-`, `border-blue-`, `border-gray-`, `border-black`

#### Scenario: Plus button renders without legacy green classes
- **WHEN** the tab bar is in structure mode
- **THEN** the `+` create button class list SHALL NOT contain `text-green-600` or `bg-green-50`

### Requirement: Tab-bar create/edit dialog uses Dialog and form atoms
When the user opens the create or edit shiftplan dialog from the tab bar, the page SHALL render a `Dialog` (variant `Auto`, width 460) whose title equals `i18n.t(Key::ShiftplanCreateTitle)` (create) or `i18n.t(Key::ShiftplanEditTitle)` (edit). The body SHALL contain a `Field` wrapping a `FormTextInput` for the shiftplan name and a `Field` wrapping a `FormCheckbox` for `is_planning` whose label equals `i18n.t(Key::ShiftplanIsPlanningLabel)`. The footer SHALL contain a `Btn` Secondary Cancel and a `Btn` Primary whose label equals `i18n.t(Key::Create)` (create) or `i18n.t(Key::Save)` (edit). The legacy `Modal` SHALL NOT be used for these dialogs.

#### Scenario: Create dialog renders Dialog with form atoms
- **WHEN** the user clicks the `+` button in structure mode
- **THEN** the rendered HTML SHALL contain a `Dialog` whose title is `i18n.t(Key::ShiftplanCreateTitle)` AND a `Field` + `FormTextInput` for the name AND a `Field` + `FormCheckbox` for `is_planning`

#### Scenario: Edit dialog pre-fills the name and is_planning
- **WHEN** the user double-clicks an existing shiftplan tab in structure mode
- **THEN** the dialog's `FormTextInput` SHALL contain that shiftplan's name AND the `FormCheckbox` SHALL match its `is_planning` value

#### Scenario: Confirm button label matches mode
- **WHEN** the dialog is in create mode
- **THEN** the footer's primary button label SHALL equal `i18n.t(Key::Create)`

#### Scenario: Confirm button label in edit mode
- **WHEN** the dialog is in edit mode
- **THEN** the footer's primary button label SHALL equal `i18n.t(Key::Save)`

#### Scenario: Cancel closes the dialog without dispatching
- **WHEN** the user clicks the Cancel button in the create or edit dialog
- **THEN** the dialog SHALL close AND no `create_shiftplan` or `update_shiftplan` API call SHALL be issued

### Requirement: Tab-bar delete-confirm dialog uses Dialog
When the user clicks the per-tab `✕` delete affordance in structure mode, the page SHALL render a confirm `Dialog` (variant `Auto`, width 420) whose title equals `i18n.t(Key::ShiftplanDeleteConfirmTitle)`. The body SHALL render `i18n.t(Key::ShiftplanDeleteConfirmBody)`. The footer SHALL contain a `Btn` Secondary Cancel and a `Btn` Danger whose label equals `i18n.t(Key::Delete)`. Clicking the Danger button SHALL dispatch the existing `api::delete_shiftplan` call and close the dialog.

#### Scenario: Delete dialog renders with Danger button
- **WHEN** the user clicks the `✕` delete affordance on a tab
- **THEN** the rendered HTML SHALL contain a `Dialog` with title `i18n.t(Key::ShiftplanDeleteConfirmTitle)` AND a footer with both a `Btn` Secondary Cancel and a `Btn` Danger Delete

#### Scenario: Cancel preserves the shiftplan
- **WHEN** the user clicks Cancel in the delete-confirm dialog
- **THEN** the dialog SHALL close AND no `api::delete_shiftplan` call SHALL be issued

#### Scenario: Confirm dispatches delete and refreshes catalog
- **WHEN** the user clicks the Danger Delete button in the delete-confirm dialog
- **THEN** the page SHALL call `api::delete_shiftplan(config, id)` and SHALL invoke `on_catalog_changed.call(None)` after the call resolves

### Requirement: Week grid uses CSS Grid scaffold with sticky time column
The week grid SHALL render as a CSS Grid container with `grid-template-columns: 76px repeat(N, minmax(140px, 1fr))` (where `N` is `7` if Sunday slots exist or `6` otherwise) and `min-width: 920px`. The first column SHALL be the time column, rendered with `position: sticky; left: 0; z-index: 2; background: var(--surface)` (or the equivalent `bg-surface` token class). Each day column SHALL render its slots via `position: absolute` placement (preserving the existing `top: y * SCALING` time math). The header row corner cell (intersection of sticky-top and sticky-left) SHALL carry `z-index: 3`; day-header cells SHALL carry `z-index: 1`.

#### Scenario: Grid container carries the column template
- **WHEN** the week grid is rendered for a week without Sunday slots
- **THEN** the grid container's inline style SHALL contain `grid-template-columns: 76px repeat(6, minmax(140px, 1fr))` AND `min-width: 920px`

#### Scenario: Grid container expands to seven columns when Sunday slots exist
- **WHEN** the week grid is rendered for a week where at least one shiftplan has slots on Sunday
- **THEN** the grid container's inline style SHALL contain `grid-template-columns: 76px repeat(7, minmax(140px, 1fr))`

#### Scenario: Time column is sticky
- **WHEN** the week grid is rendered
- **THEN** the time-column wrapper's inline style SHALL contain `position: sticky` AND `left: 0` AND a `z-index` value of `2`

#### Scenario: Header corner cell has highest z-index
- **WHEN** the week grid header row is rendered
- **THEN** the corner cell's inline style SHALL contain `z-index: 3`

#### Scenario: Horizontal overflow scrolls
- **WHEN** the week grid is rendered inside a viewport narrower than 920 px
- **THEN** the grid's parent container SHALL carry `overflow-x-auto` (or equivalent token) so the user can scroll horizontally without losing the sticky time column

### Requirement: Day header shows long name, date, and day total
Each day-column header in the week grid SHALL render against a `bg-surface-alt` background with padding `8px 10px` and a `border-bottom border-border` separator. The header content SHALL be a stacked layout containing: the long weekday name and date concatenated `{long}, {date}` in `text-[12px] font-bold` (matching reference) — OR three stacked rows where the first carries the long weekday name in `font-semibold`, the second carries the date via `i18n.format_date`, and the third carries the day's available hours from the `WEEKLY_SUMMARY_STORE` formatted as `{:.1}h` in `font-mono tabular-nums text-ink-muted text-[10px]`. The previous concatenated `<weekday>, <date> | <header>` single-line rendering SHALL NOT appear.

#### Scenario: Header has three stacked rows
- **WHEN** the day-header for any weekday is rendered
- **THEN** the header `<div>` SHALL contain three child elements: one with `font-semibold text-ink` (long weekday name), one with `text-ink-muted` (date), one with `font-mono` (day total)

#### Scenario: Day total uses tabular-nums
- **WHEN** the day-header day-total is rendered with weekly summary data loaded
- **THEN** the day-total element's class list SHALL include `font-mono` AND `tabular-nums`

#### Scenario: Day total is empty when weekly summary not loaded
- **WHEN** the day-header is rendered before `weekly_summary.data_loaded` is `true`
- **THEN** the day-total element SHALL render an empty string AND SHALL NOT crash

#### Scenario: Day total uses one decimal place
- **WHEN** the weekly summary reports `5.0` available hours for a day
- **THEN** the day-total element SHALL contain the substring `5.0h`

### Requirement: Single +/- button per cell driven by editing person
Each week-grid cell SHALL render at most one action button absolutely positioned `top: 6px; right: 6px; width: 20px; height: 20px;` whose presence and shape are driven by the editing sales person and the cell's bookings: (a) when `current_sales_person` is `None`, no button SHALL render; (b) when `current_sales_person.id` is contained in the cell's `slot.bookings` (any booking with matching `sales_person_id`), the button SHALL render as `−` with `bad`-tinted classes (`bg-bad-soft text-bad border border-bad`) and SHALL dispatch `ShiftPlanAction::RemoveUserFromSlot` on click; (c) when `current_sales_person.id` is NOT in the cell's bookings, the button SHALL render as `+` with neutral Ghost classes (`bg-surface-alt text-ink border border-border-strong hover:bg-surface-soft`) and SHALL dispatch `ShiftPlanAction::AddUserToSlot` on click. The button's `aria-label` SHALL equal `i18n.t(Key::ShiftplanCellAddTitle)` (add) or `i18n.t(Key::ShiftplanCellRemoveTitle)` (remove). When `WeekViewButtonTypes::Dropdown` is active, the same absolute slot SHALL host the `…` dropdown trigger instead, and the +/− SHALL NOT render. When `WeekViewButtonTypes::None` is active, no button SHALL render.

#### Scenario: Add button when editing person not in cell
- **WHEN** the editing person's id is NOT in any of the cell's bookings AND `button_types == AddRemove`
- **THEN** the cell SHALL render exactly one `<button>` whose visible glyph is `+` AND whose class list includes `bg-surface-alt`

#### Scenario: Remove button when editing person in cell
- **WHEN** the editing person's id IS in one of the cell's bookings AND `button_types == AddRemove`
- **THEN** the cell SHALL render exactly one `<button>` whose visible glyph is `−` AND whose class list includes `bg-bad-soft` and `text-bad`

#### Scenario: No button when no editing person
- **WHEN** `current_sales_person` is `None` AND `button_types == AddRemove`
- **THEN** the cell SHALL render no `+`/`−` button

#### Scenario: Dropdown replaces the +/− button in structure mode
- **WHEN** `button_types == Dropdown` (structure mode)
- **THEN** the cell SHALL render the `…` dropdown trigger in the absolute button slot AND SHALL NOT render a `+` or `−` button

#### Scenario: Read-only cells render no button
- **WHEN** `button_types == None` (older-than-2-weeks read-only for non-HR)
- **THEN** the cell SHALL render neither a `+`/`−` button nor a `…` dropdown trigger

#### Scenario: Add button click dispatches AddUserToSlot
- **WHEN** the user clicks the `+` button on a cell whose slot has id `S` while editing as person `P`
- **THEN** the page SHALL dispatch `ShiftPlanAction::AddUserToSlot { slot_id: S, sales_person_id: P, week, year }`

#### Scenario: Remove button click dispatches RemoveUserFromSlot
- **WHEN** the user clicks the `−` button on a cell whose slot has id `S` while editing as person `P`
- **THEN** the page SHALL dispatch `ShiftPlanAction::RemoveUserFromSlot { slot_id: S, sales_person_id: P }`

#### Scenario: Cell padding reserves space for the button
- **WHEN** any week-grid cell is rendered
- **THEN** the cell's padding SHALL resolve to `6px 32px 6px 8px` (or equivalent token classes) so chip content does not visually collide with the absolute button

### Requirement: Cell PersonChip rendering preserves tooltip and self-marker
Each week-grid cell SHALL render its booked persons via the `PersonChip` atom (one chip per booking). Each chip SHALL display the booking's `label`. The `self_added` flag SHALL be reflected by the existing `*` suffix in the label or by the `PersonChip` self prop, whichever the atom supports. The chip's background color SHALL match the booking's `background_color`. The chip text color SHALL be dark ink (per the redesign rule "PersonChip text always dark ink"). The chip SHALL contain no `<initials>` element, no two-letter abbreviation, and no avatar circle. The 500-ms-delay tooltip wiring (mousedown/touchstart timer that dispatches `TooltipAction::ShowTooltip` showing creation metadata) SHALL be preserved on a wrapping element around each chip.

#### Scenario: Cell renders one PersonChip per booking
- **WHEN** a slot has three bookings
- **THEN** the rendered cell SHALL contain exactly three `PersonChip` instances

#### Scenario: PersonChip carries no initials
- **WHEN** any PersonChip in the week grid is rendered
- **THEN** its visible text SHALL equal the booking's `label` AND SHALL NOT contain a 2-letter abbreviation that is not part of the label

#### Scenario: Self-added booking marker preserved
- **WHEN** a booking has `self_added = true`
- **THEN** the rendered chip SHALL display the label suffixed with `*` (or carry the equivalent self-marker prop the `PersonChip` atom emits)

#### Scenario: Tooltip mousedown timer still wired
- **WHEN** the user is a shiftplanner AND a chip with creation metadata is mounted
- **THEN** the chip's wrapping element SHALL still register a `mousedown` (or equivalent) handler that schedules a `TooltipAction::ShowTooltip` dispatch after 500 ms

### Requirement: Min-resources indicator uses warn-soft tokens
Each cell SHALL render a small indicator showing the booked-vs-required count formatted via `i18n.t_m(Key::ShiftplanFilledOfNeed, [("filled", ...), ("need", ...)])`. When `bookings.len() < min_resources` (understaffed), the indicator SHALL carry `bg-warn-soft text-warn` token classes. When `bookings.len() >= min_resources` (fully staffed), the indicator SHALL carry no soft tint (rendering on the cell's default surface). The legacy hard-coded inline `background-color: #ffcccc` and `background-color: #fff` SHALL NOT appear in the rendered HTML.

#### Scenario: Understaffed indicator carries warn tokens
- **WHEN** a cell renders a slot with two bookings and `min_resources = 3`
- **THEN** the min-resources indicator's class list SHALL include `bg-warn-soft` AND `text-warn`

#### Scenario: Fully-staffed indicator carries no soft tint
- **WHEN** a cell renders a slot with three bookings and `min_resources = 3`
- **THEN** the min-resources indicator's class list SHALL NOT include `bg-warn-soft`

#### Scenario: No legacy hex colors in the rendered indicator
- **WHEN** any cell is rendered
- **THEN** the rendered HTML SHALL NOT contain `#ffcccc` or `background-color: #fff` for the min-resources indicator

#### Scenario: Indicator uses the i18n format key
- **WHEN** the indicator is rendered for a slot with two bookings and `min_resources = 3`
- **THEN** the rendered text SHALL contain the substring `2/3` (whatever locale-specific separator/order `Key::ShiftplanFilledOfNeed` produces)

### Requirement: Working-hours mini overview uses auto-fit card grid with progress bar
The working-hours mini overview SHALL render as a CSS Grid with `grid-template-columns: repeat(auto-fit, minmax(180px, 1fr))` and `gap: 8px`, one card per `WorkingHoursMini` row. Each card SHALL carry `bg-surface border border-border rounded-md` with padding `10px 12px`, and `display: flex; align-items: center; gap: 10px;`. Each card SHALL contain (a) a 28×28 circular color avatar (`width:28px; height:28px; border-radius:50%; background:<sales_person.background_color>`) with no inner text, no initials, and no abbreviation, (b) a flex-1 column with the name in `text-xs font-medium`, the hours line `{actual_hours} / {dynamic_hours}h` in `font-mono text-[10px]` (color `text-warn` when `actual < target`, `text-good` when `actual >= target`), and (c) a thin progress bar (`h-[3px] bg-surface-2 rounded-[2px] mt-1 overflow-hidden`) whose filled segment uses `bg-warn` when `actual_hours < dynamic_hours` and `bg-good` when `actual_hours >= dynamic_hours`, with the filled-segment width capped at 100 %. When `show_balance` is true, the balance value `({balance_hours})` SHALL render after the hours line in `text-ink-muted`. The card whose `sales_person_id` matches `selected_sales_person_id` SHALL carry `bg-accent-soft border-accent` instead of the default surface/border.

#### Scenario: Auto-fit grid layout
- **WHEN** the mini overview is rendered with three working-hours rows
- **THEN** the container's inline style SHALL contain `grid-template-columns: repeat(auto-fit, minmax(180px, 1fr))`

#### Scenario: Color dot inline-styled
- **WHEN** any card is rendered for a working-hours row whose `background_color` is `#dbe0ff`
- **THEN** the card SHALL contain a `<span>` whose inline style includes `background-color: #dbe0ff` AND whose class list includes `rounded-full` AND whose dimensions resolve to 28×28 px

#### Scenario: Hours row uses mono tabular-nums
- **WHEN** any card is rendered
- **THEN** the hours-row element's class list SHALL include `font-mono` AND `tabular-nums`

#### Scenario: Progress bar warn color when under target
- **WHEN** a card renders a row with `actual_hours = 5.0` and `dynamic_hours = 8.0`
- **THEN** the progress-bar fill element's class list SHALL include `bg-warn`

#### Scenario: Progress bar good color when at or over target
- **WHEN** a card renders a row with `actual_hours = 8.0` and `dynamic_hours = 8.0`
- **THEN** the progress-bar fill element's class list SHALL include `bg-good`

#### Scenario: Progress bar caps at 100 percent
- **WHEN** a card renders a row with `actual_hours = 12.0` and `dynamic_hours = 8.0`
- **THEN** the progress-bar fill element's inline `width` SHALL equal `100%` (not `150%`)

#### Scenario: Selected card carries accent highlight
- **WHEN** a card's `sales_person_id` matches `selected_sales_person_id`
- **THEN** the card's class list SHALL include `bg-accent-soft` AND `border-accent`

#### Scenario: Show-balance flag renders balance suffix
- **WHEN** `show_balance` is `true` and a card renders a row with `balance_hours = -2.5`
- **THEN** the hours row SHALL contain the substring `(-2.5)`

#### Scenario: Show-balance flag hides balance suffix when false
- **WHEN** `show_balance` is `false`
- **THEN** the hours row SHALL NOT contain a parenthesized balance value

### Requirement: Booking-log table uses tokens and opacity-based deleted rows
The booking-log table SHALL use design-token classes throughout. Filter section: `bg-surface border border-border rounded-md p-3` (replacing `bg-gray-50 border border-gray-200`). Filter inputs and selects: form-input token classes (resolving to `bg-surface text-ink border border-border-strong rounded-md`). Clear-Filters button: `Btn` Secondary. Table container: `bg-surface border border-border rounded-lg overflow-hidden`. Table header: `bg-surface-alt`, header `<th>` cells use `text-ink-muted uppercase tracking-[0.04em] text-[11px] font-semibold` with padding `8px 12px`. Body rows: `border-t border-border`, with `opacity-50` applied to `<tr>` when the booking has a non-`None` `deleted` timestamp (replacing the legacy `line-through text-gray-500`). Body cells use padding `8px 12px`. The Gelöscht cell SHALL render the formatted deletion datetime when the booking is deleted, in `text-bad` color; on active rows the cell SHALL render the em-dash `—` glyph in `text-ink-muted` color. There SHALL be no badge `<span>` and no `line-through`.

#### Scenario: Deleted row carries opacity-50
- **WHEN** the table renders a booking with a non-`None` `deleted` timestamp
- **THEN** the row's class list SHALL include `opacity-50`

#### Scenario: Deleted row no longer carries line-through
- **WHEN** the table renders any booking
- **THEN** the row's class list SHALL NOT include `line-through`

#### Scenario: Deleted cell uses text-bad color
- **WHEN** the table renders a deleted booking
- **THEN** the row's Gelöscht cell's class list SHALL include `text-bad` AND its text SHALL be the formatted deletion datetime

#### Scenario: Active row's Gelöscht cell uses ink-muted em-dash
- **WHEN** the table renders an active (non-deleted) booking
- **THEN** the row's Gelöscht cell's class list SHALL include `text-ink-muted` AND its text SHALL contain the em-dash `—`

#### Scenario: No badge in Gelöscht cell
- **WHEN** the table is rendered
- **THEN** no row's Gelöscht cell SHALL contain a `<span>` whose class list includes `bg-bad-soft`

#### Scenario: No legacy gray classes in the table source
- **WHEN** the non-test source of `src/component/booking_log_table.rs` is inspected
- **THEN** it SHALL NOT contain any of `bg-gray-`, `text-gray-`, `bg-blue-`, `text-blue-`, `border-gray-`, `border-black`

### Requirement: Slot-edit modal migrated to Dialog and form atoms
The slot-edit modal SHALL render inside a `Dialog` (variant `Auto`, width 460) instead of the legacy `Modal`. The dialog title SHALL equal `i18n.t(Key::SlotNewTitle)` (when `slot_edit_type == New`) or `i18n.t(Key::SlotEditTitle)` (when `slot_edit_type == Edit`). The body SHALL contain four `Field` rows: weekday (`FormSelectInput` with one option per `Weekday`), from (token-styled time input), to (token-styled time input), min persons (token-styled integer input). The error message rendered when `has_errors` is `true` SHALL use `text-bad` (replacing `text-red-500`). The footer SHALL contain a `Btn` Secondary Cancel and a `Btn` Primary Save. Backdrop click and ESC SHALL dispatch `SlotEditAction::Cancel` (via `Dialog::on_close`). The legacy `Form`, `FormGroup`, `FormPair`, `Select`, `TimeInput`, `IntegerInput`, and `Button` from `base_components.rs` SHALL NOT appear in the rendered slot-edit DOM.

#### Scenario: Slot-edit renders inside Dialog
- **WHEN** the slot-edit is visible
- **THEN** the rendered HTML SHALL contain a `Dialog` (assert via `role="dialog"` and `aria-modal="true"`) AND SHALL NOT contain a legacy `Modal` (no `class="modal-..."` markers from `src/component/modal.rs`)

#### Scenario: Slot-edit body uses Field rows
- **WHEN** the slot-edit is visible for a New slot
- **THEN** the body SHALL contain at least four `Field` instances (weekday, from, to, min persons)

#### Scenario: Weekday Field uses FormSelectInput
- **WHEN** the slot-edit is visible
- **THEN** the weekday `Field` SHALL wrap a `FormSelectInput` whose option list contains seven entries (Monday through Sunday)

#### Scenario: Error message uses text-bad
- **WHEN** the slot-edit is visible with `has_errors = true`
- **THEN** the error message element's class list SHALL include `text-bad` AND SHALL NOT include `text-red-500`

#### Scenario: Footer Save dispatches SaveSlot
- **WHEN** the user clicks the Save button in the slot-edit dialog
- **THEN** the page SHALL dispatch `SlotEditAction::SaveSlot`

#### Scenario: Footer Cancel dispatches Cancel
- **WHEN** the user clicks the Cancel button in the slot-edit dialog
- **THEN** the page SHALL dispatch `SlotEditAction::Cancel`

#### Scenario: ESC dispatches Cancel via Dialog on_close
- **WHEN** the slot-edit dialog is open and the user presses Escape
- **THEN** the page SHALL dispatch `SlotEditAction::Cancel` via the `Dialog::on_close` callback

### Requirement: WorkingHoursMini state carries background_color
The `WorkingHoursMini` state struct SHALL include a `background_color: ImStr` field. The `WorkingHoursMiniAction::LoadWorkingHoursMini` handler SHALL populate this field from the corresponding `SalesPerson.background_color` for each row. When the loader cannot resolve a color (no matching sales person, or sales-person resource not yet loaded), the field SHALL default to a neutral gray hex (`#cccccc`) so the dot still renders. Existing fields (`sales_person_id`, `sales_person_name`, `actual_hours`, `dynamic_hours`, `balance_hours`) SHALL be unchanged.

#### Scenario: Background color populated from sales-person resource
- **WHEN** `LoadWorkingHoursMini` is dispatched and the sales-person resource is loaded
- **THEN** every entry in the resulting `WorkingHoursMini` list SHALL have a `background_color` equal to that sales person's color hex

#### Scenario: Background color defaults when sales person missing
- **WHEN** `LoadWorkingHoursMini` produces a row whose `sales_person_id` does not match any loaded sales person
- **THEN** that row's `background_color` SHALL equal `#cccccc`

#### Scenario: Existing fields unchanged
- **WHEN** the diff is inspected after the change
- **THEN** `WorkingHoursMini` SHALL still expose `sales_person_id`, `sales_person_name`, `actual_hours`, `dynamic_hours`, and `balance_hours` with their existing types

### Requirement: New i18n keys for the redesigned page
The `Key` enum SHALL include eleven new variants: `Key::ShiftplanFilledOfNeed`, `Key::ShiftplanLastWeek`, `Key::ShiftplanCellAddTitle`, `Key::ShiftplanCellRemoveTitle`, `Key::ShiftplanCreateTitle`, `Key::ShiftplanEditTitle`, `Key::ShiftplanDeleteConfirmTitle`, `Key::ShiftplanDeleteConfirmBody`, `Key::ShiftplanIsPlanningLabel`, `Key::Create`, and `Key::BookingLogDeletedTag`. Each new key SHALL have non-empty translations in all three locales (`en`, `de`, `cs`).

#### Scenario: All new keys present in all locales
- **WHEN** the i18n locale files are inspected
- **THEN** each of the eleven new keys SHALL return a non-empty string in `Locale::En`, `Locale::De`, and `Locale::Cs`

#### Scenario: Filled-of-need format substitutes placeholders
- **WHEN** `i18n.t_m(Key::ShiftplanFilledOfNeed, [("filled", "2"), ("need", "3")])` is called in any locale
- **THEN** the returned string SHALL contain the substring `2` AND the substring `3`

#### Scenario: Delete confirm body interpolates name
- **WHEN** `i18n.t_m(Key::ShiftplanDeleteConfirmBody, [("name", "Hauptplan")])` is called in any locale
- **THEN** the returned string SHALL contain the substring `Hauptplan`

### Requirement: Design tokens replace legacy classes in shiftplan sources
The non-test sources of `src/page/shiftplan.rs`, `src/component/week_view.rs`, `src/component/shiftplan_tab_bar.rs`, `src/component/working_hours_mini_overview.rs`, `src/component/booking_log_table.rs`, and `src/component/slot_edit.rs` SHALL NOT contain any of these legacy Tailwind class substrings: `bg-gray-`, `bg-white`, `text-gray-`, `text-blue-`, `text-red-`, `text-green-`, `text-orange-`, `bg-blue-`, `bg-green-`, `bg-red-`, `bg-slate-`, `border-gray-`, `border-black`. All surface, ink, border, and accent colors SHALL use design-token classes (`bg-surface`, `bg-surface-alt`, `bg-accent-soft`, `bg-warn-soft`, `bg-bad-soft`, `text-ink`, `text-ink-muted`, `text-ink-soft`, `text-warn`, `text-bad`, `text-good`, `border-border`, `border-border-strong`, `border-accent`, `text-accent`, `bg-warn`, `bg-good`, etc.).

#### Scenario: No legacy classes in shiftplan page source
- **WHEN** the non-test source of `src/page/shiftplan.rs` is inspected
- **THEN** it SHALL NOT contain any of the substrings listed in the requirement

#### Scenario: No legacy classes in week-view source
- **WHEN** the non-test source of `src/component/week_view.rs` is inspected
- **THEN** it SHALL NOT contain any of the substrings listed in the requirement

#### Scenario: No legacy classes in tab-bar source
- **WHEN** the non-test source of `src/component/shiftplan_tab_bar.rs` is inspected
- **THEN** it SHALL NOT contain any of the substrings listed in the requirement

#### Scenario: No legacy classes in mini-overview source
- **WHEN** the non-test source of `src/component/working_hours_mini_overview.rs` is inspected
- **THEN** it SHALL NOT contain any of the substrings listed in the requirement

#### Scenario: No legacy classes in booking-log source
- **WHEN** the non-test source of `src/component/booking_log_table.rs` is inspected
- **THEN** it SHALL NOT contain any of the substrings listed in the requirement

#### Scenario: No legacy classes in slot-edit source
- **WHEN** the non-test source of `src/component/slot_edit.rs` is inspected
- **THEN** it SHALL NOT contain any of the substrings listed in the requirement

### Requirement: Existing booking actions and route preserved
Clicking the `+` button SHALL still dispatch `ShiftPlanAction::AddUserToSlot { slot_id, sales_person_id, week, year }`. Clicking the `−` button SHALL still dispatch `ShiftPlanAction::RemoveUserFromSlot { slot_id, sales_person_id }`. The `ShiftPlanAction` variants `NextWeek`, `PreviousWeek`, `UpdateSalesPerson`, `CopyFromPreviousWeek`, `ToggleAvailability`, `ToggleChangeStructureMode`, `LoadWeekMessage`, `SaveWeekMessage`, `RemoveUserFromSlotDay`, and `LoadDayAggregate` SHALL still exist with identical names and payload shapes. The route `Route::ShiftPlan` SHALL render the redesigned page; no new route is added.

#### Scenario: Add dispatch payload shape unchanged
- **WHEN** the user clicks the `+` button on a cell
- **THEN** the dispatched action SHALL be a `ShiftPlanAction::AddUserToSlot` with all four existing fields populated

#### Scenario: Action variants still exist
- **WHEN** the diff is inspected after the change
- **THEN** the `ShiftPlanAction` enum SHALL still expose `NextWeek`, `PreviousWeek`, `UpdateSalesPerson`, `CopyFromPreviousWeek`, `ToggleAvailability`, `ToggleChangeStructureMode`, `LoadWeekMessage`, `SaveWeekMessage`, `RemoveUserFromSlotDay`, `LoadDayAggregate`, `AddUserToSlot`, and `RemoveUserFromSlot`

#### Scenario: Route unchanged
- **WHEN** the diff is inspected after the change
- **THEN** the router definition SHALL NOT add or remove the `Route::ShiftPlan` variant
