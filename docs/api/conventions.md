# API Conventions

This file collects conventions that apply across individual endpoints.
For concrete endpoint definitions, see the feature docs.

## Request/response format

- **Content-Type:** `application/json` in both directions.
- **Character encoding:** UTF-8.

## HTTP methods

| Method | Semantics | Idempotent? |
| --- | --- | --- |
| GET | Read | Yes |
| POST | Create or command | No |
| PUT | Full update / upsert | Yes |
| PATCH | Partial update | No |
| DELETE | Soft delete (sets the `deleted` column) | Yes |

**Important note on DELETE:** Shifty does soft delete, not hard delete.
Readers filter `deleted IS NULL`. A `DELETE` call sets the column but
does not physically remove the row.

## URL structure

- **Kebab-case:** `/sales-person`, `/billing-period`, not
  `/salesPerson`.
- **Resources in plural:** `/bookings`, `/absences`.
- **Sub-resources:** `/sales-person/{id}/employee-work-details`.

**[To verify]** — uniformity is not 100% across the repo (some routes
are singular, some plural). For new routes: stay consistent.

## UUIDs

All entity IDs are UUIDs, transmitted as hyphenated strings:

```
"550e8400-e29b-41d4-a716-446655440000"
```

Not: comma-separated, without dashes, or as a byte array.

## Time & date

- **Date only:** ISO 8601, `YYYY-MM-DD`.
- **Datetime:** ISO 8601, `YYYY-MM-DDTHH:MM:SSZ` (UTC — **[To verify]**).

No Unix timestamps. No locale-dependent formatting in the API.

## Enums

Wire format: string with the exact Rust variant name.

```json
{ "category": "SickLeave" }
```

Not: lowercase, snake_case, or numeric ID.

## Optional fields

Rust `Option<T>` becomes `null` or is omitted entirely —
**[To verify]** how serde is configured in Shifty (`skip_serializing_if`
or `null`?).

## Error format

Errors are returned as JSON:

```json
{
  "error": "ValidationError",
  "message": "sales_person_id must not be empty",
  "details": null
}
```

**[To verify]** — exact field set from `error_handler`.

## Auth header

- **`mock_auth`:** no header required.
- **`oidc`:** bearer token:
  ```
  Authorization: Bearer <token>
  ```

## Transactions from the client's perspective

One API call = one backend transaction. The client sees an **atomic**
outcome: either success with full effect, or an error with no partial
effect.

There are no explicit client-side transactions ("BEGIN"/"COMMIT" across
multiple requests). If you need a composite op, that's a case for a
dedicated backend endpoint that bundles the ops internally.

## Pagination

**[To verify]** — current state. Many endpoints appear to return a full
list. For large datasets, offset-based pagination would be the next
step.

## Rate limiting

**[To verify]** — whether rate limits are configured. Probably not
today.

## API versioning

- **No URL prefix.** Currently `/booking`, not `/v1/booking`.
- **Breaking changes** are communicated via the SemVer backend version.
  A second client should display the backend version in the UI or log.
- **For DTO changes:** additive changes (new field) stay compatible;
  removed fields trigger a major bump.

## Idempotency keys

**[To verify]** — whether idempotency-header support exists. Probably
not today — retry behaviour has to be made safe by the client (a POST
retry can produce duplicates).

## Long-polling / WebSockets

**No.** Shifty is request-response. No WebSocket, no SSE. For live
updates, the client must poll periodically.
