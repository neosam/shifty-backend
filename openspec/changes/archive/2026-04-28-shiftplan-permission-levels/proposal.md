## Why

The existing sales person to shiftplan assignment system is binary: a person is either assigned to a shiftplan (can be booked) or not (cannot be booked). This applies uniformly regardless of who performs the booking. Shift planners need the ability to book people into shiftplans that those people cannot self-assign to, enabling controlled scheduling while restricting self-service options.

## What Changes

- Add a `permission_level` field to the `sales_person_shiftplan` join table with values `available` (self-service allowed) and `planner_only` (only shiftplanners can book)
- Modify eligibility checks to be context-aware: when a shiftplanner books someone, `planner_only` assignments are treated as eligible; when a person books themselves, only `available` assignments are eligible
- Modify `get_bookable_sales_persons` to accept context so it can return the appropriate list depending on who is requesting
- When a person is booked into a `planner_only` shiftplan by a shiftplanner, the person can see the booking but cannot add or remove it themselves
- The permissive default remains: no assignments means full access everywhere

## Capabilities

### New Capabilities
- `shiftplan-permission-levels`: Permission level distinction (`available` vs `planner_only`) on sales person to shiftplan assignments, with role-aware eligibility checks

### Modified Capabilities

## Impact

- **Database**: New column `permission_level` on `sales_person_shiftplan` table (migration needed, default `available` for existing rows)
- **DAO layer**: `SalesPersonShiftplanDao` methods need to return/accept permission level information
- **Service layer**: `is_eligible()` and `get_bookable_sales_persons()` need authentication context parameter; `BookingService::create` and `BookingService::delete` need adjusted permission logic
- **REST layer**: Updated DTOs to include permission level; endpoint responses may differ based on caller role
- **Frontend**: Dropdown/selection UI needs to reflect permission levels (out of scope for backend change)
