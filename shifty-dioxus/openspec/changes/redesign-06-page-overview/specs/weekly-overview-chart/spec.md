## MODIFIED Requirements

### Requirement: Stacked bar chart displays paid and volunteer hours per week
The system SHALL render a stacked bar chart using HTML flex columns (one column per ISO week of the displayed year) where the bottom segment of each bar represents paid hours and the segment above it represents volunteer hours. Bar segments SHALL use design tokens applied via the `style` attribute: paid uses `var(--accent)` (current week) or a designed dimmer accent shade for non-current weeks, and volunteer uses `var(--ink-muted)` with `opacity: 0.35`.

#### Scenario: Chart renders one bar per week
- **WHEN** the weekly overview page loads data for a year
- **THEN** the chart container renders one column per `WeeklySummary` entry, with bar segment heights proportional to `paid_hours` and `volunteer_hours` against the chart's max-hours scale

#### Scenario: Empty weeks render no segments
- **WHEN** a week has zero paid hours and zero volunteer hours
- **THEN** the column for that week SHALL contain no paid/volunteer divs (only the absolute-positioned required-line indicator if applicable)

#### Scenario: Bar colors use tokens, not legacy hex
- **WHEN** the chart renders
- **THEN** the paid segment SHALL include `background: var(--accent)` (for the current-week bar) or the designed dimmer accent (for non-current bars), AND the volunteer segment SHALL include `background: var(--ink-muted); opacity: 0.35`, AND no legacy hex (`#3B82F6`, `#10B981`, `#EF4444`, `#e5e7eb`, `#6b7280`) SHALL appear in the chart source

### Requirement: Required-hours reference line overlays each bar
The system SHALL render the `required_hours` value for each week as an absolute-positioned dashed horizontal indicator inside that week's bar column. The indicator SHALL use `border-top: 1.5px dashed var(--bad)` (red, dashed) so it visually contrasts with the bars and clearly signals the required-hours target on a per-week basis.

#### Scenario: Required indicator renders per bar
- **WHEN** a bar column is rendered for any week with `required_hours > 0`
- **THEN** the column SHALL contain an absolute-positioned div with `border-top: 1.5px dashed var(--bad)` whose `top` is computed as `(1 - required_hours / max_hours) * 100%`

#### Scenario: Required indicator uses bad token, not muted
- **WHEN** any bar's required indicator is rendered
- **THEN** its color SHALL resolve via `var(--bad)`, NOT `var(--ink-muted)` and NOT a hardcoded red hex like `#EF4444`

### Requirement: Chart legend identifies visual elements
The page SHALL display a legend above the chart, rendered in HTML, with three entries: a paid swatch (12×12 px, `var(--accent)`), a volunteer swatch (12×12 px, `var(--ink-muted)` with `opacity: 0.4`), and a required-hours swatch (12×2 px horizontal bar in `var(--bad)`). Legend labels SHALL be translated via the i18n system.

#### Scenario: Legend rendered as HTML above the bars
- **WHEN** the chart card is rendered
- **THEN** an HTML container with the legend's three entries SHALL appear above the bars container

#### Scenario: Required-line swatch uses bad token
- **WHEN** the legend renders the required-hours entry
- **THEN** its visual swatch SHALL be a thin horizontal bar (height ≈ 2 px) using `var(--bad)`, mirroring the per-bar dashed indicator's color

#### Scenario: Volunteer swatch uses ink-muted with reduced opacity
- **WHEN** the legend renders the volunteer entry
- **THEN** its visual swatch SHALL use `var(--ink-muted)` with `opacity: 0.4`, NOT `var(--good)` or `#10B981`

#### Scenario: Legend displays in current locale
- **WHEN** the user's locale is German
- **THEN** the legend shows translated labels for paid hours, volunteer hours, and required hours

### Requirement: Week labels on X-axis
The system SHALL display exactly five mono-font week labels along the X-axis below the bars, using the format `<WeekShort> <n>` for weeks 1, 13, 26, 39, and 52 (where `<WeekShort>` is the locale-specific short prefix from `Key::WeekShort`). Labels SHALL be distributed evenly via flex `space-between`. Labels SHALL render in `var(--ink-muted)` color and a small font size (≈ 10 px).

#### Scenario: Five locale-prefixed labels appear
- **WHEN** the chart renders in German locale
- **THEN** the X-axis row SHALL contain exactly the labels `KW 1`, `KW 13`, `KW 26`, `KW 39`, `KW 52`, in that order

#### Scenario: Labels use mono font and ink-muted color
- **WHEN** any X-axis label renders
- **THEN** its container SHALL include classes or styles resolving to a monospace font and `color: var(--ink-muted)`

## ADDED Requirements

### Requirement: Current-week bar visually emphasized
When the chart renders for a year that includes today's ISO week, the bar representing the current ISO week SHALL render at full opacity (`opacity: 1`) while all other bars SHALL render at `opacity: 0.85`. The current-week bar's paid segment SHALL use `var(--accent)` while non-current bars' paid segments SHALL use a designed dimmer accent shade. When the displayed year does not include today's ISO week, no bar SHALL receive the current-week treatment and all bars SHALL render at full opacity.

#### Scenario: Current week is full opacity, others dimmed
- **WHEN** the chart renders the current calendar year and the current ISO week is week 17
- **THEN** the bar for week 17 SHALL render with `opacity: 1`, AND every other bar SHALL render with `opacity: 0.85`

#### Scenario: Current bar uses accent, others use designed dimmer accent
- **WHEN** a bar represents the current ISO week of the displayed year
- **THEN** its paid segment's `background` SHALL be `var(--accent)`, AND non-current bars' paid segments SHALL use a designed dimmer accent shade (e.g. `#7787e8`)

#### Scenario: No highlight when navigated away from current year
- **WHEN** the chart renders a year that is not the current calendar year
- **THEN** no bar SHALL render with `opacity: 0.85`, AND no bar's paid segment SHALL use `var(--accent)` — all bars use the dimmer accent shade at full opacity

### Requirement: Bar columns are clickable and route to the week view
Each bar column SHALL be a clickable element that navigates to the corresponding shiftplan week URL (`/shiftplan/{year}/{week}`) when activated. Each bar column SHALL include a tooltip (`title`) summarizing the week's paid, volunteer, and required hours.

#### Scenario: Bar click triggers navigation
- **WHEN** a user clicks on any bar column
- **THEN** the page navigates to `/shiftplan/{year}/{week}` for that bar's `(year, week)`

#### Scenario: Bar exposes a tooltip
- **WHEN** any bar is rendered
- **THEN** the bar's outermost element SHALL include a `title` attribute containing the week number, paid hours, volunteer hours, and required hours

## REMOVED Requirements

### Requirement: Y-axis scales dynamically to data
**Reason**: The redesigned chart matches the reference design which has no visible Y-axis labels or grid lines. Bar heights are scaled internally against the max of `(paid + volunteer)` and `required_hours` per week, but no numeric Y-axis is rendered.
**Migration**: Per-bar `required_hours` is communicated by the per-bar dashed indicator (in `var(--bad)`); per-bar absolute values are available via the bar's `title` tooltip.

### Requirement: Responsive hybrid layout
**Reason**: The redesigned chart uses HTML flex columns with `flex: 1` so bars naturally fill the container width. Horizontal scrolling and minimum-bar-width logic are no longer needed. Bars become thinner on narrow viewports until a small minimum width is reached, then naturally stay readable via spacing.
**Migration**: No user action needed; the new layout is automatically responsive.
