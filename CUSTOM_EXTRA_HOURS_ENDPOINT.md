# Custom Extra Hours by Sales Person Endpoint

## Overview

A new REST endpoint has been implemented to retrieve custom extra hours for a specific sales person.

## Endpoint Details

**URL:** `GET /custom-extra-hours/by-sales-person/{sales_person_id}`

**Parameters:**
- `sales_person_id` (UUID): The ID of the sales person to retrieve custom extra hours for

**Response:**
- **200 OK**: Returns an array of `CustomExtraHoursTO` objects
- **403 Forbidden**: Insufficient permissions
- **404 Not Found**: Sales person not found
- **500 Internal Server Error**: Server error

## Authentication & Authorization

The endpoint implements proper authentication and authorization:

1. **HR Privilege**: Users with HR privilege can access custom extra hours for any sales person
2. **Sales Person Access**: A sales person can only access their own custom extra hours
3. **Access Control**: The service verifies that the requesting user is either:
   - Has HR privilege, OR
   - Is the same sales person whose data is being requested

## Implementation Details

### REST Layer
- Added new route `/by-sales-person/{sales_person_id}` to the custom extra hours router
- Implemented `get_by_sales_person_id` endpoint function with proper OpenAPI documentation
- Uses existing `CustomExtraHoursTO` transfer object for response

### Service Layer
- Leverages existing `get_by_sales_person_id` method in `CustomExtraHoursService`
- Authentication is handled using `tokio::join!` to check both HR privilege and sales person verification
- Uses `hr_permission.or(sales_person_permission)?` pattern for authorization

### DAO Layer
- Uses existing `find_by_sales_person_id` method in `CustomExtraHoursDao`
- SQLite implementation uses default trait implementation that filters by sales person ID
- Properly handles the many-to-many relationship between custom extra hours and sales persons

## Testing

Comprehensive tests have been added to verify:

1. **Successful retrieval**: HR users and sales persons can retrieve custom extra hours
2. **Permission denied**: Users without proper permissions are rejected
3. **Sales person specific access**: Sales persons can only access their own data
4. **HR access**: HR users can access any sales person's data

## Usage Example

```bash
# As HR user - can access any sales person's data
GET /custom-extra-hours/by-sales-person/123e4567-e89b-12d3-a456-426614174000

# As sales person - can only access own data
GET /custom-extra-hours/by-sales-person/123e4567-e89b-12d3-a456-426614174000
```

## Response Format

```json
[
  {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "name": "Overtime",
    "description": "Additional overtime hours",
    "modifies_balance": true,
    "assigned_sales_person_ids": ["123e4567-e89b-12d3-a456-426614174000"],
    "created": "2023-10-01T12:00:00",
    "deleted": null,
    "$version": "456e7890-e89b-12d3-a456-426614174000"
  }
]
```