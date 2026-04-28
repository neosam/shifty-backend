## ADDED Requirements

### Requirement: Detail header renders color circle, name, type pill, target, year nav, and actions
The Employee Details page SHALL render a header row containing, in order: a 32 px circular element whose `background-color` is the employee's `sales_person.background_color`, the employee's name styled as `text-xl font-semibold text-ink`, a type pill rendered using the `PersonChip` atom (with the `--accent-soft` hex when `sales_person.is_paid` is true and the `--warn-soft` hex otherwise), the target hours formatted as `{expected:.0} h` in monospace tabular-numbers, a year-navigation cluster (previous-year `NavBtn`, year display in mono, next-year `NavBtn`), a primary-variant `Btn` labeled by `Key::OtherHours`, and a secondary-variant `Btn` labeled by `Key::More` followed by a `▾` glyph that opens the existing `DropdownTrigger` menu. The color circle SHALL contain no text (no initials, no abbreviation).

#### Scenario: Header includes color circle and name
- **WHEN** the page is rendered for an employee with `background_color = "#dbe0ff"` and `name = "Lena"`
- **THEN** the header HTML SHALL contain an element with inline style `background-color: #dbe0ff` resolving to a 32 px circle, AND the text `Lena` styled with classes resolving to `text-xl`, `font-semibold`, and `text-ink`

#### Scenario: Color circle has no inner text
- **WHEN** the header is rendered
- **THEN** the color circle element SHALL have no text content

#### Scenario: Type pill is Bezahlt for paid sales persons
- **WHEN** the page is rendered for a sales person with `is_paid = true` in German locale
- **THEN** the header SHALL contain an element using `PersonChip` styling (carrying class `person-pill`) with the visible text `Bezahlt`

#### Scenario: Type pill is Freiwillig for volunteer sales persons
- **WHEN** the page is rendered for a sales person with `is_paid = false` in German locale
- **THEN** the header SHALL contain an element using `PersonChip` styling with the visible text `Freiwillig`

#### Scenario: Year navigation uses NavBtn atoms
- **WHEN** the header is rendered
- **THEN** the previous-year and next-year buttons SHALL be `NavBtn` instances with classes resolving to `border-border-strong` and `font-mono`, AND no legacy `border-2 border-solid border-black` button SHALL appear in the header

#### Scenario: Sonstige Stunden button opens the extra-hours modal
- **WHEN** the user clicks the primary-variant button labeled `Key::OtherHours`
- **THEN** the page SHALL set state that opens the `ExtraHoursModal` for the current employee

### Requirement: Three-column sub-grid for overview, contracts/histogram, and extra hours
Below the header, the page SHALL render a CSS Grid with `grid-template-columns: repeat(auto-fit, minmax(280px, 1fr))` containing three direct children, in order: a Gesamtansicht column (`Key::OverallHeading`), an Arbeitsverträge + Stunden pro Woche column (`Key::WorkDetailsHeading` over `Key::WorkingHoursPerWeekHeading`), and a Zusatzarbeit column (`Key::ExtraHoursHeading`).

#### Scenario: Grid uses auto-fit minmax
- **WHEN** the detail body is rendered
- **THEN** the body's grid wrapper SHALL include an inline style or class resolving to `grid-template-columns: repeat(auto-fit, minmax(280px, 1fr))`

#### Scenario: Grid contains exactly three columns
- **WHEN** the detail body is rendered
- **THEN** the grid wrapper SHALL contain three direct child elements, one per column

### Requirement: Gesamtansicht column lists primary and dim TupleRows
The Gesamtansicht column SHALL render, in order, three primary `TupleRow`s (`Balance`, `Overall`, `Required`), a divider with class `border-t border-border`, and then a stack of `dim={true}` `TupleRow`s for the category breakdown (`CategoryShiftplan`, `CategoryExtraWork`, `CategoryVacation`, `CategorySickLeave`, `CategoryHolidays`, `CategoryUnpaidLeave`, `CategoryVolunteerWork`, `CarryoverBalance`) plus one row per `custom_extra_hours` entry. When `show_vacation = true`, the column SHALL additionally render a `VacationDaysLabel` row showing `<vacation_days> / <vacation_entitlement>` and a `VacationCarryoverLabel` row. Each row's value element SHALL use mono tabular-numbers formatting with two decimal places.

#### Scenario: Primary rows render at the top
- **WHEN** the Gesamtansicht column is rendered
- **THEN** the first three rows SHALL be `TupleRow` elements with non-dim styling (no `text-ink-muted` on the label) for Balance, Overall, and Required in that order

#### Scenario: Dim rows follow the divider
- **WHEN** the Gesamtansicht column is rendered with breakdown values
- **THEN** the breakdown rows SHALL render with `dim={true}` (label class resolving to `text-ink-muted`)

#### Scenario: Values use mono tabular-numbers
- **WHEN** any TupleRow value renders a numeric hours figure
- **THEN** the value's element SHALL include classes resolving to `font-mono` and `tabular-nums` and the formatted text SHALL include exactly two decimal digits

### Requirement: Contracts column renders cards and an add-contract button
The Arbeitsverträge + Stunden pro Woche column SHALL render, in order: one card per `EmployeeWorkDetails` entry, an "Add contract" button labeled by `Key::AddWorkDetailsLabel`, the `EmployeeWeeklyHistogram` component, and the inline week-detail panel when a week is selected. Each contract card SHALL be a clickable button-like element showing `<from> – <to>` left and `<expected> h/Woche` right. Clicking a card SHALL open the `ContractModal` in edit mode for that contract; clicking the add button SHALL open the `ContractModal` in new mode.

#### Scenario: One card per contract in chronological order
- **WHEN** the column is rendered for an employee with two contracts (`from=2025-01-01` and `from=2026-03-01`)
- **THEN** the column SHALL contain two card elements with the earlier `from` date first

#### Scenario: Card click opens edit modal
- **WHEN** the user clicks a contract card
- **THEN** the page SHALL set state that opens the `ContractModal` with `form_type = Edit` for that contract

#### Scenario: Add-contract button opens new modal
- **WHEN** the user clicks the add-contract button
- **THEN** the page SHALL set state that opens the `ContractModal` with `form_type = New`

### Requirement: Inline week-detail panel renders below the histogram
When the `selected_week` state is `Some((year, week))`, the page SHALL render an inline panel below the histogram showing the selected week's `from`–`to` date range, a summary line `<overall>/<expected> h` in mono tabular-numbers, and one row per `WorkingHoursDay` entry in `working_hours_by_week[selected]`'s `days` array. Each day row SHALL show the date, the localized weekday, the localized category name (`category.to_i18n_key()`), and the hours in mono tabular-numbers. When the same bar is clicked twice (or the panel's close affordance is activated), the panel SHALL close and `selected_week` SHALL become `None`. The panel SHALL render hours per day with category — NOT time blocks.

#### Scenario: Panel renders day rows with category labels
- **WHEN** the user selects a week containing 5 `WorkingHoursDay` entries
- **THEN** the panel SHALL render exactly 5 day rows, each containing the date, the category translation, and the hours formatted with two decimal places

#### Scenario: Panel does not render time blocks
- **WHEN** the panel renders any day row
- **THEN** the row SHALL NOT contain any `from–to` time-of-day pair (e.g. `09:00–13:30`); only date, weekday, category, and hours

#### Scenario: Re-clicking the same bar closes the panel
- **WHEN** `selected_week = Some((2026, 17))` and the user clicks the bar for week 17
- **THEN** `selected_week` SHALL become `None` and the panel SHALL not render

### Requirement: Zusatzarbeit column lists categorized extra-hours entries
The Zusatzarbeit column SHALL render the `ExtraHours` list grouped by category in this order: vacation, holidays, sick leave, extra work, unavailable, unpaid leave, volunteer, then one group per `CustomExtraHoursDefinition`. Each group SHALL begin with an `<h3>` heading using the category's i18n key and styled with `text-xs uppercase tracking-wide font-semibold text-ink-muted`. Each entry row SHALL render the date (left), the optional description (below the date in `text-ink-muted text-xs`), the value in mono tabular-numbers (`<amount> {Hours}`), and a danger-variant `Btn` with the trash icon for delete. Categories with no entries SHALL NOT render their heading.

#### Scenario: Entries render with token classes
- **WHEN** any extra-hours entry row renders
- **THEN** the row SHALL include classes resolving to `border-b border-border` and `text-ink`/`text-ink-muted`, AND it SHALL NOT contain `text-gray-500`, `text-gray-600`, or `border-gray-200`

#### Scenario: Empty category headings are omitted
- **WHEN** an employee has no vacation entries
- **THEN** the Zusatzarbeit column SHALL NOT contain a vacation category heading

#### Scenario: Delete button uses Danger variant
- **WHEN** an extra-hours row's delete button is rendered
- **THEN** the button SHALL be a `Btn` with `variant: BtnVariant::Danger` carrying classes `text-bad` and `border-bad`

### Requirement: Mehr ▾ dropdown contains year-toggle entries only
The `Mehr ▾` dropdown SHALL be the existing `DropdownTrigger` whose entries are reduced to the year-toggle actions: a `Key::ShowFullYearLabel` entry (visible only when the displayed year is the current year) and a `Key::ShowUntilNowLabel` entry (visible only when the displayed year is the current year). The legacy `AddEntry` and `AddWorkDetailsLabel` entries SHALL NOT appear in the dropdown — `Key::OtherHours` is its own header button, and adding a contract is the dedicated button at the bottom of the contracts column.

#### Scenario: Dropdown excludes redundant entries
- **WHEN** the dropdown is opened
- **THEN** the rendered entries SHALL NOT contain text matching `i18n.t(Key::AddEntry)` or `i18n.t(Key::AddWorkDetailsLabel)`

### Requirement: Design tokens replace legacy classes in employee-details sources
The non-test sources of `src/page/employee_details.rs` and `src/component/employee_view.rs` SHALL NOT contain any of these legacy Tailwind classes: `bg-gray-100`, `bg-gray-200`, `bg-white`, `text-gray-500`, `text-gray-600`, `text-gray-700`, `text-gray-900`, `text-blue-600`, `text-blue-800`, `text-red-500`, `text-red-600`, `text-green-500`, `bg-blue-600`, `bg-green-600`, `bg-red-600`, `border-black`, `border-gray-200`, `border-gray-300`. All surface, ink, border, and accent colors SHALL use design-token classes.

#### Scenario: No legacy classes in employee-details page source
- **WHEN** the non-test source of `src/page/employee_details.rs` is inspected
- **THEN** it SHALL NOT contain any of the substrings listed in the requirement

#### Scenario: No legacy classes in employee-view component source
- **WHEN** the non-test source of `src/component/employee_view.rs` is inspected
- **THEN** it SHALL NOT contain any of the substrings listed in the requirement
