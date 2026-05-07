# working-hours-mini-overview Specification

## Purpose
TBD - created by archiving change working-hours-overview-layout-toggle. Update Purpose after archive.
## Requirements
### Requirement: Component renders cards layout by default

The `WorkingHoursMiniOverview` component SHALL render a CSS-grid card layout when its `layout` prop is `Cards` (or omitted, with `Cards` being the default). Each card SHALL contain the employee's color dot, name, "actual / target h" line and a horizontal progress bar.

The card grid SHALL use `repeat(auto-fit, minmax(180px, 1fr))` as its grid-template-columns value.

#### Scenario: Cards layout renders one card per employee

- **WHEN** the component is rendered with three `WorkingHoursMini` rows and `layout: Cards`
- **THEN** the rendered DOM SHALL contain three card elements
- **AND** each card SHALL show the employee name and the "actual / target h" string

#### Scenario: Cards layout uses auto-fit grid

- **WHEN** the component is rendered with `layout: Cards`
- **THEN** the grid container SHALL declare `grid-template-columns: repeat(auto-fit, minmax(180px, 1fr))`

### Requirement: Component renders table layout when requested

The `WorkingHoursMiniOverview` component SHALL render a tabular layout when its `layout` prop is `Table`. The table SHALL contain a header row with columns labelled `Employee`, `Actual`, `Target`, `Difference`, `Utilization`, one body row per employee, and a footer row with column totals.

The `Difference` cell SHALL display the signed delta `actual - target` with one decimal and an `h` suffix, prefixed with `+` when non-negative.

The `Utilization` cell SHALL contain a horizontal progress bar (capped at 100%) and a percent label rounded to the nearest integer.

The footer SHALL sum `actual` and `target` over all visible rows and display the totals in the corresponding columns; the `Utilization` cell in the footer SHALL be empty.

#### Scenario: Table renders header, body and footer

- **WHEN** the component is rendered with two rows and `layout: Table`
- **THEN** the DOM SHALL contain a `<table>` element with a `<thead>`, a `<tbody>` with two body rows, and one footer row labelled `Total`

#### Scenario: Difference column is signed and color-coded

- **WHEN** an employee has `actual_hours = 22.0` and `dynamic_hours = 20.0`
- **THEN** the row's difference cell SHALL display the text `+2.0h`
- **AND** the cell SHALL carry the `text-good` class

#### Scenario: Difference column for under-target employee

- **WHEN** an employee has `actual_hours = 15.0` and `dynamic_hours = 20.0`
- **THEN** the row's difference cell SHALL display the text `-5.0h`
- **AND** the cell SHALL carry the `text-warn` class

#### Scenario: Utilization column shows progress and percent

- **WHEN** an employee has `actual_hours = 5.0` and `dynamic_hours = 10.0`
- **THEN** the row's utilization cell SHALL contain a progress bar element with `width: 50%`
- **AND** SHALL contain the text `50%`

#### Scenario: Footer row totals all visible rows

- **WHEN** the rows are `(actual=5.0, target=10.0)` and `(actual=12.0, target=10.0)`
- **THEN** the footer SHALL display `17.0h` in the `Actual` column and `20.0h` in the `Target` column
- **AND** the difference cell SHALL display `+2.0h`

### Requirement: Layout toggle is rendered above the overview

The Shiftplan page SHALL render a segmented toggle directly above the `WorkingHoursMiniOverview`, with two options labelled by the i18n keys for "Cards" and "Table". The active option SHALL be styled distinctly from the inactive option using only design tokens.

The active option of the toggle SHALL match the layout currently rendered by the component.

#### Scenario: Toggle reflects current layout

- **WHEN** the page is rendered with the working-hours layout set to `Cards`
- **THEN** the `Cards` button SHALL carry the active style
- **AND** the `Table` button SHALL carry the inactive style

#### Scenario: Clicking the toggle switches the layout

- **WHEN** the user clicks the `Table` option of the toggle
- **THEN** the working-hours overview SHALL re-render in the table layout
- **AND** the `Table` button SHALL become the active option

### Requirement: Layout choice is persisted across reloads

The page SHALL persist the chosen layout via the UI-preferences service so the same layout is used on subsequent renders within the same browser. After a reload, the overview SHALL render in the previously chosen layout without user interaction.

If the persisted value is missing or unrecognized, the layout SHALL default to `Cards`.

#### Scenario: First visit defaults to cards

- **WHEN** the page is loaded with no stored working-hours layout
- **THEN** the overview SHALL render in the cards layout
- **AND** the `Cards` toggle option SHALL be active

#### Scenario: Stored value is restored after reload

- **WHEN** the user has selected the table layout in a previous session
- **AND** the page is reloaded
- **THEN** the overview SHALL render in the table layout on first paint
- **AND** the `Table` toggle option SHALL be active

#### Scenario: Selecting a layout writes it to storage

- **WHEN** the user clicks the `Table` option of the toggle
- **THEN** the UI-preferences service SHALL store the value `table` for the working-hours layout key

### Requirement: Both layouts share data, sort and selection behavior

Both the cards and the table layout SHALL render the same set of `WorkingHoursMini` rows in the same order (alphabetical by `sales_person_name`). Both layouts SHALL emit the existing double-click handler with the row's `sales_person_id`. Both layouts SHALL apply the existing accent styling to the row whose `sales_person_id` matches `selected_sales_person_id`.

#### Scenario: Same row order in both layouts

- **WHEN** the rows are provided in arbitrary order with names `Charlie`, `Alice`, `Bob`
- **AND** the component is rendered once with `layout: Cards` and once with `layout: Table`
- **THEN** both renderings SHALL emit the rows in the order `Alice`, `Bob`, `Charlie`

#### Scenario: Selected row is highlighted in table layout

- **WHEN** the component is rendered with `layout: Table` and a `selected_sales_person_id` matching one of the rows
- **THEN** the matching `<tr>` SHALL carry the accent styling token

#### Scenario: Double-click in table emits the row's id

- **WHEN** the user double-clicks a `<tr>` in the table layout
- **THEN** the `on_dbl_click` handler SHALL be invoked with that row's `sales_person_id`

