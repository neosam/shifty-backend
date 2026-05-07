# ui-preferences Specification

## Purpose
TBD - created by archiving change working-hours-overview-layout-toggle. Update Purpose after archive.
## Requirements
### Requirement: Working-hours layout getter returns the stored value

The UI-preferences service SHALL expose a `get_working_hours_layout` function that returns the stored layout choice for the working-hours overview. The function SHALL read from `localStorage` under a stable, namespaced key.

If the stored value is the string `cards`, the function SHALL return the `Cards` variant.

If the stored value is the string `table`, the function SHALL return the `Table` variant.

If the key is missing, the stored value is unrecognized, or `localStorage` is unavailable, the function SHALL return the `Cards` variant.

#### Scenario: Stored value cards

- **WHEN** `localStorage` contains the value `cards` under the working-hours layout key
- **THEN** `get_working_hours_layout()` SHALL return `Cards`

#### Scenario: Stored value table

- **WHEN** `localStorage` contains the value `table` under the working-hours layout key
- **THEN** `get_working_hours_layout()` SHALL return `Table`

#### Scenario: Missing key falls back to cards

- **WHEN** `localStorage` does not contain the working-hours layout key
- **THEN** `get_working_hours_layout()` SHALL return `Cards`

#### Scenario: Unknown value falls back to cards

- **WHEN** `localStorage` contains the value `gallery` under the working-hours layout key
- **THEN** `get_working_hours_layout()` SHALL return `Cards`

#### Scenario: localStorage unavailable falls back to cards

- **WHEN** `localStorage` is unavailable (private mode, disabled, or non-browser environment)
- **THEN** `get_working_hours_layout()` SHALL return `Cards` without panicking

### Requirement: Working-hours layout setter writes the value

The UI-preferences service SHALL expose a `set_working_hours_layout(layout)` function that writes the layout choice to `localStorage` under the same key the getter reads.

The function SHALL serialize `Cards` as the string `cards` and `Table` as the string `table`.

If `localStorage` is unavailable, the function SHALL be a no-op and SHALL NOT panic.

#### Scenario: Setter writes cards string

- **WHEN** `set_working_hours_layout(Cards)` is called
- **THEN** `localStorage` SHALL contain the value `cards` under the working-hours layout key

#### Scenario: Setter writes table string

- **WHEN** `set_working_hours_layout(Table)` is called
- **THEN** `localStorage` SHALL contain the value `table` under the working-hours layout key

#### Scenario: Round-trip preserves the value

- **WHEN** `set_working_hours_layout(Table)` is called
- **AND** `get_working_hours_layout()` is called immediately afterwards
- **THEN** `get_working_hours_layout()` SHALL return `Table`

### Requirement: Storage key is namespaced

The UI-preferences service SHALL prefix every key it reads or writes with a stable, app-specific namespace so it cannot collide with unrelated keys written by other code that may share the same `localStorage`.

The namespace prefix SHALL be `shifty.ui.`.

#### Scenario: Working-hours layout key carries the namespace

- **WHEN** `set_working_hours_layout(Table)` is called
- **THEN** the written key SHALL begin with `shifty.ui.`

