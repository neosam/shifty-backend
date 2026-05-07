## ADDED Requirements

### Requirement: Stacked bar chart displays paid and volunteer hours per week
The system SHALL render an SVG stacked bar chart where each bar represents one calendar week. The bottom segment of each bar SHALL represent paid hours and the top segment SHALL represent volunteer hours.

#### Scenario: Chart renders with correct bar heights
- **WHEN** the weekly overview page loads data for a year
- **THEN** each week is represented by a stacked bar where the paid-hours segment height is proportional to `paid_hours` and the volunteer-hours segment height is proportional to `volunteer_hours`

#### Scenario: Empty weeks show no bar
- **WHEN** a week has zero paid hours and zero volunteer hours
- **THEN** no bar is rendered for that week

### Requirement: Required-hours reference line overlays the bars
The system SHALL render a polyline connecting the `required_hours` value for each week, overlaying the bar chart. This line SHALL be visually distinct (red color) from the bars.

#### Scenario: Reference line tracks required hours
- **WHEN** the chart renders
- **THEN** a continuous line connects the required-hours values across all weeks at the correct Y-axis position

### Requirement: Y-axis scales dynamically to data
The system SHALL compute the Y-axis maximum from the data as the greater of `max(paid_hours + volunteer_hours)` and `max(required_hours)` across all weeks, rounded up to the next multiple of 10. Horizontal grid lines SHALL be drawn at regular intervals.

#### Scenario: Y-axis adapts to data range
- **WHEN** the maximum value across all weeks is 37 hours
- **THEN** the Y-axis maximum is set to 40 and grid lines are drawn at 0, 10, 20, 30, 40

### Requirement: Chart legend identifies visual elements
The system SHALL display a legend showing the meaning of the paid-hours color, volunteer-hours color, and required-hours line. Legend labels SHALL be translated via the i18n system.

#### Scenario: Legend displays in current locale
- **WHEN** the user's locale is German
- **THEN** the legend shows translated labels for paid hours, volunteer hours, and required hours

### Requirement: Responsive hybrid layout
On viewports where all bars fit at a readable width, the chart SHALL scale to fill the container. When bars would become narrower than the minimum width (approximately 16px per bar step), the SVG SHALL maintain the minimum bar width and the container SHALL scroll horizontally.

#### Scenario: Desktop viewport shows full chart
- **WHEN** the viewport is wide enough to display all bars at or above minimum width
- **THEN** the chart fills the available width without horizontal scrolling

#### Scenario: Mobile viewport enables horizontal scroll
- **WHEN** the viewport is too narrow to display all bars at minimum width
- **THEN** the container scrolls horizontally and bars maintain readable width

### Requirement: Chart is positioned above the existing table
The system SHALL render the chart between the year navigation controls and the data table on the weekly overview page.

#### Scenario: Chart placement on page
- **WHEN** the weekly overview page renders with loaded data
- **THEN** the chart appears after the year navigation buttons and before the weekly data table

### Requirement: Week labels on X-axis
The system SHALL display week numbers along the X-axis below the bars. Labels SHALL use a compact format (e.g., "1", "2", ... "52").

#### Scenario: X-axis labels match week numbers
- **WHEN** the chart renders weeks 1 through 52
- **THEN** each bar has a corresponding week number label below it

### Requirement: i18n translation keys for chart
The system SHALL add translation keys for chart-related labels to all three locales (English, German, Czech).

#### Scenario: All locales have chart translations
- **WHEN** the chart renders in any supported locale (En, De, Cs)
- **THEN** the legend and axis labels display in the correct language
