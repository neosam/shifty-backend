## Why

The backend has renamed the `/shiftplan` endpoint to `/shiftplan-catalog` to better distinguish the catalog CRUD endpoints from the shiftplan view endpoints (`/shiftplan-info`).

## What Changes

- Rename the API URL in `get_all_shiftplans()` from `/shiftplan` to `/shiftplan-catalog`

## Capabilities

### New Capabilities

### Modified Capabilities
- `shiftplan-catalog`: API endpoint URL updated from `/shiftplan` to `/shiftplan-catalog`

## Impact

- `src/api.rs`: Single URL string change in `get_all_shiftplans()`
