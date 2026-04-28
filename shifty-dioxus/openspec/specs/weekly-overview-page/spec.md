# weekly-overview-page Specification

## Purpose
TBD - created by archiving change redesign-06-page-overview. Update Purpose after archive.
## Requirements
### Requirement: Page layout uses tokenized stacked card structure
The Weekly Overview page SHALL render its content as a vertically stacked sequence of cards inside a centered container. The page SHALL render the redesigned `TopBar` above all content, the page heading using `Key::WeeklyOverviewTitle`, a year-navigation row, a chart card, and a table card — in that order. Card surfaces SHALL use the `bg-surface` token, `border-border` for the border, and `rounded-md` for corners. No legacy `bg-white`, `bg-gray-*`, `text-gray-*`, `text-green-500`, `text-red-500`, or `text-yellow-700` Tailwind classes SHALL appear in the page source.

#### Scenario: Page wrapper is centered with tokenized heading
- **WHEN** the page is rendered with loaded data
- **THEN** the main content container SHALL include classes resolving to horizontal centering (`mx-auto`) and a bounded max width, AND the `<h1>` heading SHALL contain `i18n.t(Key::WeeklyOverviewTitle)` styled with token-based ink color

#### Scenario: Cards use surface and border tokens
- **WHEN** the chart card and the table card are rendered
- **THEN** each card's outermost element SHALL include classes resolving to `bg-surface`, `border-border`, and `rounded-md` (or equivalent token-based class names) — AND no `bg-white` or `bg-gray-100` class SHALL appear on either card

#### Scenario: No legacy color classes in page source
- **WHEN** the non-test source of `src/page/weekly_overview.rs` is inspected
- **THEN** it SHALL NOT contain any of `bg-gray-100`, `bg-white`, `text-gray-500`, `text-gray-600`, `text-green-500`, `text-red-500`, `text-yellow-700`, or `border-black`

### Requirement: Year navigation row uses NavBtn atoms
The year-navigation row SHALL render as a flex row containing a previous-year `NavBtn`, the current year value in monospace font, and a next-year `NavBtn`. Each `NavBtn` SHALL receive a glyph (`‹` for previous, `›` for next), an `aria_label` translated via the i18n system, and an `on_click` handler that dispatches `WeeklyOverviewPageAction::PreviousYear` or `WeeklyOverviewPageAction::NextYear` to the page coroutine. The wrapping row SHALL include `print:hidden` so the navigation does not appear in printed output.

#### Scenario: Both nav buttons are NavBtn atoms
- **WHEN** the year-navigation row is rendered
- **THEN** the previous-year and next-year buttons SHALL be instances of the `NavBtn` atom (rendered with classes resolving to `border-border-strong` and `font-mono`), AND no legacy `border-2 border-solid border-black` button SHALL remain in the page source

#### Scenario: Year display uses mono font
- **WHEN** the year-navigation row renders the current year value
- **THEN** the year value SHALL appear inside an element with classes resolving to `font-mono` and tokenized ink color

#### Scenario: Aria labels translated
- **WHEN** the page renders in German locale
- **THEN** the previous-year `NavBtn` SHALL receive an `aria_label` resolving to the translation of `Key::PreviousYear`, AND the next-year `NavBtn` SHALL receive `Key::NextYear`

#### Scenario: Print hides the year nav
- **WHEN** the page is rendered
- **THEN** the wrapping container of the previous-year and next-year buttons SHALL include the `print:hidden` class

### Requirement: Table card renders all weeks of the displayed year
The table card SHALL render one row per `WeeklySummary` entry in the loaded data, in the order provided. The table SHALL NOT filter, paginate, or otherwise limit the visible weeks. Users need full-year visibility to scan the whole year for staffing patterns and to navigate to any week directly from the table.

#### Scenario: Full year renders all 52 weeks
- **WHEN** the page loads weekly summaries for a 52-week year
- **THEN** the table SHALL render exactly 52 data rows, one per week, in week-number order

#### Scenario: Different displayed year still renders all weeks
- **WHEN** the displayed year is 2024 but the current calendar year is 2026
- **THEN** the table SHALL render all weeks of 2024 (no filtering by year-match)

### Requirement: Current-week table row tinted with accent-soft
When a table row's `(year, week)` matches the current calendar year and current ISO week, the row SHALL include a class resolving to `bg-accent-soft`. All other rows SHALL render with the default surface background.

#### Scenario: Current-week row receives accent tint
- **WHEN** the table renders the current ISO week of the current calendar year
- **THEN** that row's `<tr>` element SHALL include the class `bg-accent-soft`

#### Scenario: Non-current rows are not tinted
- **WHEN** the table renders any week other than the current week
- **THEN** that row's `<tr>` element SHALL NOT include the class `bg-accent-soft`

#### Scenario: Different year never tints
- **WHEN** the displayed year does not equal the current calendar year
- **THEN** no `<tr>` element in the table SHALL include the class `bg-accent-soft`

### Requirement: Diff column collapses to two color tiers
The diff column SHALL display `available_hours - required_hours` formatted with two decimals and a leading sign. The text color SHALL be `text-warn` when the diff is negative (deficit), `text-good` when the diff is positive (surplus), and the default `text-ink` when the diff is exactly zero. The legacy thresholds (greater-than-20-hour deficit rendering red, greater-than-7-hour deficit rendering yellow) SHALL be removed.

#### Scenario: Surplus is good
- **WHEN** a week has `available_hours = 38.0` and `required_hours = 35.0`
- **THEN** the diff cell SHALL render `+ 3.00` (or the locale-appropriate format) inside an element with class `text-good`

#### Scenario: Deficit is warn
- **WHEN** a week has `available_hours = 30.0` and `required_hours = 35.0`
- **THEN** the diff cell SHALL render `- 5.00` inside an element with class `text-warn`

#### Scenario: Large deficit no longer red
- **WHEN** a week has `available_hours = 10.0` and `required_hours = 35.0`
- **THEN** the diff cell SHALL render `- 25.00` inside an element with class `text-warn` (NOT `text-bad` or `text-red-500`)

#### Scenario: Zero diff is neutral
- **WHEN** a week has `available_hours == required_hours`
- **THEN** the diff cell SHALL render `0.00` (or equivalent) without `text-good` or `text-warn` classes

### Requirement: Absences row restyled with token classes
For any visible week that has at least one entry in `sales_person_absences`, a secondary row SHALL appear directly below the week's row, spanning all columns, listing each absence as `<name>: <hours> <hours-short>`. The absences row SHALL use `text-ink-muted text-xs` styling and SHALL NOT itself receive the `bg-accent-soft` tint.

#### Scenario: Absences row uses token classes
- **WHEN** a week has at least one absence and is rendered in the visible window
- **THEN** the absences row's `<td>` element SHALL include classes resolving to `text-ink-muted` and `text-xs`, AND it SHALL NOT contain the legacy `text-gray-600` class

#### Scenario: Absences row never tinted
- **WHEN** the current week has at least one absence
- **THEN** the week's primary row SHALL include `bg-accent-soft`, AND the immediately-following absences row SHALL NOT include `bg-accent-soft`

### Requirement: New i18n keys for year navigation and week prefix
The system SHALL provide three new i18n keys: `PreviousYear`, `NextYear`, and `WeekShort`. Each key SHALL have translations in all three supported locales (English, German, Czech). `PreviousYear` and `NextYear` provide the aria labels for the year-navigation `NavBtn` atoms. `WeekShort` provides the short prefix used in chart X-axis labels.

#### Scenario: All three locales have translations
- **WHEN** the page renders in any supported locale
- **THEN** `i18n.t(Key::PreviousYear)`, `i18n.t(Key::NextYear)`, and `i18n.t(Key::WeekShort)` SHALL each return non-empty locale-specific strings

#### Scenario: WeekShort produces locale-aware chart labels
- **WHEN** the chart renders X-axis labels in German locale
- **THEN** each visible label SHALL be the concatenation of `i18n.t(Key::WeekShort)` (which is `"KW"`), a space, and the week number

