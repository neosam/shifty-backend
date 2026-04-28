## ADDED Requirements

### Requirement: Slot cell crops with fixed height and internal chip scroll
Each `WeekCellSlot` outer `<div>` SHALL keep its time-derived absolute placement (`top` and `height` computed from `(slot.from_hour() - day_start) * SCALING` and `(slot.to_hour() - slot.from_hour()) * SCALING`) AND SHALL additionally carry the class `overflow-hidden` so chip content cannot paint outside the slot's box, regardless of the number of bookings.

Inside the slot box, the rendered DOM SHALL contain three siblings (no other layout-bearing elements between them):
1. an absolutely positioned min-resources pill (`top: 6px; left: 8px; pointer-events: none`),
2. an absolutely positioned chip area whose inset reserves space for the pill on the left and the action button on the right (`inset: 6px 32px 6px 38px`), carrying `flex flex-wrap content-start overflow-y-auto overflow-x-hidden`,
3. an absolutely positioned action area at `top: 6px; right: 6px` hosting the +/- button or the `…` dropdown trigger (the existing `cell_button_classes` output is preserved).

The chip area's children SHALL be one `WeekCellChip` (or its tooltip-wrapping element) per booking, and nothing else. The min-resources pill and the action button SHALL NOT be children of the chip area.

#### Scenario: Slot box carries overflow-hidden
- **WHEN** any `WeekCellSlot` is rendered
- **THEN** the outer `<div>`'s class list SHALL contain `overflow-hidden`

#### Scenario: Chip area carries the scroll classes
- **WHEN** any `WeekCellSlot` is rendered with one or more bookings
- **THEN** the chip-area `<div>` SHALL contain the classes `flex`, `flex-wrap`, `content-start`, `overflow-y-auto`, AND `overflow-x-hidden`

#### Scenario: Chip area inset reserves left and right space
- **WHEN** any `WeekCellSlot` is rendered
- **THEN** the chip-area `<div>`'s inline `style` SHALL contain `inset: 6px 32px 6px 38px` (or an equivalent four-value inset producing the same offsets)

#### Scenario: Chip area contains only chips
- **WHEN** a slot has three bookings
- **THEN** the chip-area `<div>` SHALL contain exactly three booking elements AND SHALL NOT contain the min-resources pill or the action button

#### Scenario: Many bookings do not overflow the slot
- **WHEN** a slot whose time-derived height is less than the natural stack height of its bookings is rendered
- **THEN** the outer slot box's painted area SHALL NOT extend below its computed `height` (verified structurally by `overflow-hidden` on the outer and `overflow-y-auto` on the chip area)

## MODIFIED Requirements

### Requirement: Single +/- button per cell driven by editing person
Each week-grid cell SHALL render at most one action button absolutely positioned `top: 6px; right: 6px; width: 20px; height: 20px;` whose presence and shape are driven by the editing sales person and the cell's bookings: (a) when `current_sales_person` is `None`, no button SHALL render; (b) when `current_sales_person.id` is contained in the cell's `slot.bookings` (any booking with matching `sales_person_id`), the button SHALL render as `−` with `bad`-tinted classes (`bg-bad-soft text-bad border border-bad`) and SHALL dispatch `ShiftPlanAction::RemoveUserFromSlot` on click; (c) when `current_sales_person.id` is NOT in the cell's bookings, the button SHALL render as `+` with neutral Ghost classes (`bg-surface-alt text-ink border border-border-strong hover:bg-surface-soft`) and SHALL dispatch `ShiftPlanAction::AddUserToSlot` on click. The button's `aria-label` SHALL equal `i18n.t(Key::ShiftplanCellAddTitle)` (add) or `i18n.t(Key::ShiftplanCellRemoveTitle)` (remove). When `WeekViewButtonTypes::Dropdown` is active, the same absolute slot SHALL host the `…` dropdown trigger instead, and the +/− SHALL NOT render. When `WeekViewButtonTypes::None` is active, no button SHALL render. The button (or dropdown trigger) SHALL be a sibling of the chip-area `<div>`, NOT a child of it, so that its visibility is independent of the chip area's scroll position.

#### Scenario: Add button when editing person not in cell
- **WHEN** the editing person's id is NOT in any of the cell's bookings AND `button_types == AddRemove`
- **THEN** the cell SHALL render exactly one `<button>` whose visible glyph is `+` AND whose class list includes `bg-surface-alt`

#### Scenario: Remove button when editing person in cell
- **WHEN** the editing person's id IS in one of the cell's bookings AND `button_types == AddRemove`
- **THEN** the cell SHALL render exactly one `<button>` whose visible glyph is `−` AND whose class list includes `bg-bad-soft` and `text-bad`

#### Scenario: No button when no editing person
- **WHEN** `current_sales_person` is `None`
- **THEN** the cell SHALL render no `+`/`−` button

#### Scenario: Action button is a sibling of the chip area
- **WHEN** any `WeekCellSlot` renders an action button or dropdown trigger
- **THEN** that button element SHALL be a direct child of the outer slot `<div>` AND SHALL NOT be a descendant of the chip-area `<div>`

### Requirement: Min-resources indicator uses warn-soft tokens
Each cell SHALL render a small indicator showing the booked-vs-required count formatted via `i18n.t_m(Key::ShiftplanFilledOfNeed, [("filled", ...), ("need", ...)])`. The indicator SHALL be absolutely positioned at `top: 6px; left: 8px` with `pointer-events: none` so it does not intercept clicks intended for the chip area or the action button. When `bookings.len() < min_resources` (understaffed), the indicator SHALL carry `bg-warn-soft text-warn` token classes. When `bookings.len() >= min_resources` (fully staffed), the indicator SHALL carry no soft tint (rendering on the cell's default surface). The legacy hard-coded inline `background-color: #ffcccc` and `background-color: #fff` SHALL NOT appear in the rendered HTML. The indicator SHALL be a sibling of the chip-area `<div>`, NOT a child of it.

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

#### Scenario: Indicator is absolutely positioned and pointer-events none
- **WHEN** any min-resources indicator is rendered
- **THEN** its inline style SHALL place it at `top: 6px; left: 8px` AND SHALL include `pointer-events: none`

#### Scenario: Indicator is a sibling of the chip area
- **WHEN** any cell with a min-resources indicator is rendered
- **THEN** the indicator element SHALL be a direct child of the outer slot `<div>` AND SHALL NOT be a descendant of the chip-area `<div>`
