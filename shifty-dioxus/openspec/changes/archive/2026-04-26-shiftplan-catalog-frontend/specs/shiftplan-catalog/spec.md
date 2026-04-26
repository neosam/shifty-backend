## ADDED Requirements

### Requirement: Load shiftplan catalog
The system SHALL load all available shiftplans from the backend endpoint `GET /shiftplan` when the shiftplan page is opened.

#### Scenario: Successful catalog fetch
- **WHEN** the shiftplan page is loaded
- **THEN** all shiftplans (`ShiftplanTO`) are fetched from the backend and available for selection

#### Scenario: Empty catalog
- **WHEN** the backend returns no shiftplans
- **THEN** no tab is displayed and the WeekView shows no data

### Requirement: First shiftplan selected automatically
The system SHALL automatically select the first shiftplan from the list as the active one on load.

#### Scenario: Initial selection
- **WHEN** the shiftplan catalog has been loaded and at least one shiftplan exists
- **THEN** the first shiftplan in the list is set as the active shiftplan

### Requirement: Tab bar for shiftplan selection
The system SHALL display a tab bar above the WeekView showing all available shiftplans as tabs.

#### Scenario: Display tabs
- **WHEN** the shiftplan catalog has been loaded
- **THEN** a tab is shown for each shiftplan using the `name` field as the label

#### Scenario: Highlight active tab
- **WHEN** a shiftplan is selected
- **THEN** the corresponding tab is visually highlighted as active

#### Scenario: Switch tab
- **WHEN** the user clicks on a different tab
- **THEN** that shiftplan becomes active and the week view reloads with data from the selected shiftplan

### Requirement: Load week view with shiftplan_id
The system SHALL load the week view via `GET /shiftplan-info/{shiftplan_id}/{year}/{week}` using the `shiftplan_id` of the active tab.

#### Scenario: Load data for selected shiftplan
- **WHEN** a shiftplan is active and week/year are selected
- **THEN** `GET /shiftplan-info/{shiftplan_id}/{year}/{week}` is called with the active `shiftplan_id`

#### Scenario: Reload on shiftplan change
- **WHEN** the user switches the active shiftplan via the tab bar
- **THEN** the week view reloads with the new `shiftplan_id`

### Requirement: Slots with shiftplan_id
The system SHALL support the field `shiftplan_id: Option<Uuid>` on `SlotTO`.

#### Scenario: Ignore slot without shiftplan_id
- **WHEN** a loaded slot has `shiftplan_id == None`
- **THEN** a warning is logged and the slot is not displayed

#### Scenario: New slot receives shiftplan_id
- **WHEN** a new slot is created
- **THEN** the `shiftplan_id` of the currently active shiftplan is set

### Requirement: API functions for shiftplan catalog
The system SHALL provide API functions to retrieve the shiftplan catalog.

#### Scenario: Fetch all shiftplans
- **WHEN** `get_all_shiftplans(config)` is called
- **THEN** `GET /shiftplan` is called and a list of `ShiftplanTO` is returned

### Requirement: ShiftplanTO REST type
The system SHALL define the `ShiftplanTO` type with the fields `id`, `name`, `is_planning`, `deleted`, and `version` in the frontend.

#### Scenario: Deserialization
- **WHEN** the backend returns a `ShiftplanTO` JSON response
- **THEN** it is correctly deserialized into the Rust struct
