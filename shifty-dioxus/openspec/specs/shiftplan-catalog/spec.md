# Capability: Shiftplan Catalog

## Purpose
Provide access to the shiftplan catalog for retrieving available shiftplans.

## Requirements

### Requirement: API functions for shiftplan catalog
The system SHALL provide API functions to retrieve the shiftplan catalog from `GET /shiftplan-catalog`.

#### Scenario: Fetch all shiftplans
- **WHEN** `get_all_shiftplans(config)` is called
- **THEN** `GET /shiftplan-catalog` is called and a list of `ShiftplanTO` is returned
