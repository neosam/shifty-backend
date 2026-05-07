# my-shifts-page Specification

## Purpose
TBD - created by archiving change redesign-05-page-myshifts. Update Purpose after archive.
## Requirements
### Requirement: Centered week-card stack
The My Shifts page SHALL render its content as a vertically stacked list of week cards inside a centered container with a maximum width of 760 px on viewports above the mobile breakpoint, and full available width below it. The page SHALL continue to render the redesigned `TopBar` above the card stack and the page heading using the `Key::MyShifts` translation.

#### Scenario: Page is centered with bounded width on desktop
- **WHEN** the My Shifts page is rendered on a viewport wider than 760 px
- **THEN** the main content container SHALL have classes resolving to `max-width: 760px`, horizontal centering (`mx-auto`), and the redesigned `TopBar` SHALL appear above it

#### Scenario: Page heading uses i18n key
- **WHEN** the page is rendered with any active locale
- **THEN** the heading element SHALL contain the text returned by `i18n.t(Key::MyShifts)` and SHALL be styled with design-token ink color (no hardcoded `text-gray-*`)

### Requirement: Per-week card with header and full week of day rows
For every ISO week in the loaded range that has at least one block, the page SHALL render a single card containing a header row and seven day rows (Monday through Sunday). The header SHALL display, on the left, the week label `KW <week> · <date range>` (using `Key::WeekLabel` for the label fragment and `i18n.format_date` for both ends of the range), and on the right the total hours for the week formatted with one decimal place using a monospace, tabular-numbers font. Each day row SHALL be laid out as a CSS Grid with three columns: day label (mono, 110 px on desktop / 80 px on mobile), shift items (flexible), day total hours (mono, right-aligned, 60 px on desktop / 50 px on mobile). Cards SHALL NOT be collapsible.

#### Scenario: Week with shifts renders one card with seven day rows
- **WHEN** a week in the loaded range contains one or more `BlockTO` records
- **THEN** the page SHALL render exactly one card for that week, AND the card SHALL contain seven day-row elements in Monday-through-Sunday order

#### Scenario: Header shows week number and date range
- **WHEN** a card is rendered for ISO week 17 of 2026 spanning April 20 to April 26
- **THEN** the header SHALL contain the substring `KW 17` (or the localized week-label equivalent) followed by a separator and the localized date strings for April 20 and April 26

#### Scenario: Header total hours uses mono one-decimal format
- **WHEN** the blocks in a card sum to 12.5 hours
- **THEN** the right-hand value of the header SHALL render the text `12.5` inside an element carrying classes resolving to `font-family: monospace` and `font-variant-numeric: tabular-nums`

#### Scenario: Empty weeks are omitted
- **WHEN** a week-key exists in the grouped blocks map but the week's block list is empty
- **THEN** no card SHALL be rendered for that week

#### Scenario: All weekdays present even when only some have shifts
- **WHEN** a week has shifts only on Monday and Friday
- **THEN** the card SHALL still render seven day rows, with non-empty rows for Monday and Friday and empty-state rows for the other five days

#### Scenario: Cards never expose a collapse toggle
- **WHEN** any card is rendered
- **THEN** the card markup SHALL NOT contain an interactive control that collapses or hides the day rows

### Requirement: Shift item formatted as time range plus area badge
Each shift item inside a day row SHALL render as a `HH:MM–HH:MM` time range in monospace tabular-numbers font, immediately followed by an area badge implemented as the existing `PersonChip` atom. The badge SHALL use the sales person's `background_color` as its pastel background and the sales person's `name` as its text. When a block has no associated sales person, the badge SHALL render in `PersonChip`'s dashed-border, no-color fallback variant containing a `-` placeholder.

#### Scenario: Time range uses mono tabular numbers
- **WHEN** a block has `from = 09:00` and `to = 13:30`
- **THEN** the rendered time range SHALL contain the text `09:00–13:30` inside an element with classes resolving to `font-family: monospace` and `font-variant-numeric: tabular-nums`

#### Scenario: Badge background mirrors sales-person color
- **WHEN** a block's sales person has `background_color = "#ffd6c1"` and `name = "Lena"`
- **THEN** the rendered badge SHALL include an inline style `background-color: #ffd6c1`, the class `person-pill`, and the visible text `Lena`

#### Scenario: Missing sales person uses dashed fallback
- **WHEN** a block has `sales_person: None`
- **THEN** the rendered badge SHALL NOT contain the class `person-pill` and SHALL contain the classes `border-dashed`, `border-border-strong`, and the placeholder text `-`

#### Scenario: Multiple shifts on one day stack vertically
- **WHEN** a single day contains two `BlockTO` records
- **THEN** the day row's middle column SHALL stack the two shift items in column-flex order, AND the day label and day total in the same row SHALL remain top-aligned

### Requirement: Empty-day placeholder and per-day hour total
When a day has no shifts, the day row's middle column SHALL render a single em-dash (`—`) styled with `text-ink-muted`, and the right-hand hours column SHALL render `0.0` in the mono tabular-numbers format. When a day has at least one shift, the right-hand hours column SHALL render the sum of the day's block durations with one decimal place using the same mono tabular-numbers format.

#### Scenario: Empty day shows muted em-dash
- **WHEN** Tuesday of a rendered card has no blocks
- **THEN** the Tuesday row's middle column SHALL contain a single `—` glyph inside an element with class `text-ink-muted`

#### Scenario: Empty day hour total is zero
- **WHEN** a day has no blocks
- **THEN** its hours column SHALL render the text `0.0` in mono tabular-numbers format

#### Scenario: Day with shifts shows summed hours
- **WHEN** a day contains two blocks of duration 3.5 h and 2.0 h respectively
- **THEN** its hours column SHALL render the text `5.5`

### Requirement: Design tokens replace legacy gray classes
The My Shifts page SHALL NOT carry any of the following legacy Tailwind utility classes: `bg-gray-100`, `bg-gray-200`, `bg-white`, `text-gray-500`, `text-gray-600`, `text-gray-800`, `text-red-600`. All surface, ink, and border colors SHALL use the design-token aliases (`bg-surface`, `bg-surface-alt`, `border-border`, `text-ink`, `text-ink-soft`, `text-ink-muted`, `text-bad`, etc.) introduced in the `design-tokens` capability.

#### Scenario: Card surfaces use tokens
- **WHEN** any card is rendered
- **THEN** the card markup SHALL contain classes resolving to `background: var(--surface)` and a border using `border-border`, AND it SHALL NOT contain `bg-gray-100`, `bg-gray-200`, or `bg-white`

#### Scenario: Loading state uses ink-muted token
- **WHEN** the blocks resource is still loading
- **THEN** the loading indicator SHALL use a `text-ink-muted` class and SHALL NOT use `text-gray-500`

#### Scenario: Error state uses bad token
- **WHEN** the blocks resource resolves to an error
- **THEN** the error indicator SHALL use a class resolving to `color: var(--bad)` and SHALL NOT use `text-red-600`

### Requirement: No-shifts state preserved
When the loaded range contains zero blocks across every week, the page SHALL render a single message styled with `text-ink-muted` containing the text returned by `i18n.t(Key::NoShiftsFound)` and SHALL NOT render any week cards.

#### Scenario: All weeks empty shows the no-shifts message
- **WHEN** `loader::load_blocks(...)` returns an empty list
- **THEN** the page SHALL render exactly one element containing `i18n.t(Key::NoShiftsFound)` and SHALL NOT render any card markup

### Requirement: Existing data loading and route preserved
The page SHALL continue to load data via `loader::load_blocks(config, current_year, current_week, current_year + offset, current_week + 10)` (carrying over the existing 10-weeks-ahead window) and SHALL continue to be reachable at the existing route. The shape of `BlockTO` and the `loader::load_blocks` signature SHALL NOT change as part of this redesign.

#### Scenario: Loader is invoked with the same window
- **WHEN** the My Shifts page mounts in week W of year Y
- **THEN** `loader::load_blocks` SHALL be called with the start tuple `(Y, W)` and an end tuple ten ISO weeks later (with year roll-over handled identically to the current implementation)

#### Scenario: Backend contract unchanged
- **WHEN** the redesigned page is built
- **THEN** the build SHALL succeed without any modification to `BlockTO` or to the `loader::load_blocks` function signature

