# Shiftplan Day View

## Purpose
Provide a day-oriented view of the shiftplan that aggregates all shiftplans as columns for a single selected weekday, allowing users to inspect and edit bookings across plans for that day from a single screen.

## Requirements

### Requirement: View mode toggle between week and day view
The shiftplan page SHALL provide a toggle control that switches between week view and day view. The default view mode SHALL be week view.

#### Scenario: User toggles to day view
- **WHEN** the user clicks the "Day" toggle button on the shiftplan page
- **THEN** the week view is replaced by the day aggregate view showing all shiftplans as columns for the selected day

#### Scenario: User toggles back to week view
- **WHEN** the user clicks the "Week" toggle button while in day view
- **THEN** the day view is replaced by the standard week view for the current week and selected shiftplan

### Requirement: Smart default day selection on toggle
When switching to day view, the system SHALL pre-select the current weekday if viewing the current ISO week, or Monday if viewing any other week.

#### Scenario: Toggle to day view in current week
- **WHEN** the user toggles to day view and the displayed week matches the current ISO week
- **THEN** the selected day SHALL be today's weekday

#### Scenario: Toggle to day view in a different week
- **WHEN** the user toggles to day view and the displayed week does not match the current ISO week
- **THEN** the selected day SHALL be Monday

### Requirement: Day button bar for direct day selection
The day view SHALL display a button bar with one button per weekday (Monday through Saturday, optionally Sunday). The currently selected day SHALL be visually highlighted.

#### Scenario: User clicks a day button
- **WHEN** the user clicks a weekday button in the day button bar
- **THEN** the day view SHALL load and display data for the selected day

#### Scenario: Sunday button visibility
- **WHEN** no shiftplan has slots defined for Sunday
- **THEN** the Sunday button SHALL NOT be displayed in the day button bar

#### Scenario: Sunday button visible when slots exist
- **WHEN** at least one shiftplan has slots defined for Sunday
- **THEN** the Sunday button SHALL be displayed in the day button bar

### Requirement: Arrow navigation between days
The day view SHALL provide left and right arrow buttons for sequential day navigation. Navigation SHALL wrap across week boundaries.

#### Scenario: Navigate to next day
- **WHEN** the user clicks the right arrow and the current day is not the last visible day
- **THEN** the selected day SHALL advance to the next weekday

#### Scenario: Navigate to previous day
- **WHEN** the user clicks the left arrow and the current day is not Monday
- **THEN** the selected day SHALL go back to the previous weekday

#### Scenario: Navigate right from last day of week
- **WHEN** the user clicks the right arrow and the current day is Saturday (or Sunday if visible)
- **THEN** the week SHALL advance to the next week and the selected day SHALL be Monday

#### Scenario: Navigate left from Monday
- **WHEN** the user clicks the left arrow and the current day is Monday
- **THEN** the week SHALL go back to the previous week and the selected day SHALL be Saturday (or Sunday if Sunday slots exist in that week)

### Requirement: All shiftplans displayed as columns
In day view, the system SHALL display one column per shiftplan, with the shiftplan name as the column header. All active shiftplans for the selected day SHALL be shown simultaneously.

#### Scenario: Multiple shiftplans on a day
- **WHEN** the day view loads for a day with slots in three shiftplans
- **THEN** three columns SHALL be displayed, each headed with the respective shiftplan name

#### Scenario: Shiftplan with no slots on selected day
- **WHEN** a shiftplan has no slots defined for the selected day
- **THEN** that shiftplan SHALL still appear as a column but with no slot entries

### Requirement: Slot display with booking information
Each slot within a plan column SHALL display the time range, booked persons with their background colors, and the min-resources indicator. Understaffed slots SHALL be visually highlighted.

#### Scenario: Slot with bookings displayed
- **WHEN** a slot has bookings assigned
- **THEN** the slot SHALL show each booked person's name with their background color and the resource count (e.g., "2/3")

#### Scenario: Understaffed slot highlighting
- **WHEN** a slot has fewer bookings than its min_resources value
- **THEN** the slot SHALL be displayed with a warning background color

### Requirement: Booking add and remove in day view
Users SHALL be able to add and remove bookings from slots in the day view using the same interaction pattern as the week view (add/remove buttons or dropdown depending on permissions).

#### Scenario: Add a booking in day view
- **WHEN** the user adds a sales person to a slot in the day view
- **THEN** a booking SHALL be created via the existing booking API and the day view SHALL refresh to show the updated state

#### Scenario: Remove a booking in day view
- **WHEN** the user removes a booking from a slot in the day view
- **THEN** the booking SHALL be deleted via the existing booking API and the day view SHALL refresh to show the updated state

### Requirement: Tab bar hidden in day view
The shiftplan tab bar (used for selecting individual plans in week view) SHALL NOT be displayed when in day view, since all plans are already visible as columns.

#### Scenario: Tab bar visibility in day view
- **WHEN** the view mode is set to day view
- **THEN** the shiftplan tab bar SHALL be hidden

#### Scenario: Tab bar visibility in week view
- **WHEN** the view mode is set to week view
- **THEN** the shiftplan tab bar SHALL be displayed as usual

### Requirement: Time column in day view
The day view SHALL display a sticky time column on the left side, showing hour labels, consistent with the week view's TimeView component.

#### Scenario: Time column alignment
- **WHEN** the day view is displayed
- **THEN** a time column SHALL be shown on the left with hour labels aligned to the slot time ranges across all plan columns
