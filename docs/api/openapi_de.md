# OpenAPI & Swagger-UI

## Wo das Schema lebt

Das OpenAPI-Schema wird über [utoipa](https://docs.rs/utoipa)
zur Compile-Zeit aus Rust-Code generiert:

- **Route-Annotationen** — `#[utoipa::path(...)]` auf jedem
  REST-Handler.
- **Schema-Ableitung** — `#[derive(ToSchema)]` auf jedem DTO in
  `rest-types/src/`.
- **Zusammensetzen** — `ApiDoc` in `rest/src/lib.rs` oder ähnlich
  aggregiert alle annotierten Handler und Schemas.

## Swagger-UI

Im laufenden Backend erreichbar unter:

```
http://<host>:<port>/swagger-ui
```

**[Zu prüfen]** — genauer Pfad.

Für Zweit-Client-Devs ist Swagger-UI die verbindliche Referenz für
Endpoint-Signaturen, DTOs und Beispiele.

## Authentifizierung im API

- **`mock_auth`-Build:** Keine echte Auth — jeder Request wird als
  Admin behandelt. Nur für Dev.
- **`oidc`-Build:** OpenID Connect. Bearer-Token im `Authorization`-Header.

Details: [`../architecture/04-auth.md`](../architecture/04-auth.md).

## Error-Mapping

Alle REST-Handler routen `ServiceError` durch einen zentralen
`error_handler`-Wrapper. Die Mapping-Tabelle:

| ServiceError | HTTP-Status | Wann |
| --- | --- | --- |
| `Unauthorized` | 401 | Kein / ungültiger Auth-Token |
| `Forbidden` | 403 | Auth ok, aber Rolle reicht nicht |
| `NotFound` | 404 | Entity existiert nicht (oder soft-deleted) |
| `ValidationError(...)` | 400 | Bad Request Body / Params |
| `Conflict(...)` | 409 | Duplikat, Overlap, Race |
| `InternalError(...)` | 500 | Alles andere |

**[Zu prüfen]** — genaue Enum-Varianten in `service/src/lib.rs` oder
`service_impl/src/lib.rs`.

## DTO-Konventionen

- **Naming:** DTOs enden auf `TO` (Transport Object), z.B. `BookingTO`,
  `SalesPersonTO`.
- **Wire-Format:** JSON.
- **UUIDs:** Als String (hyphenated).
- **Dates:** ISO 8601 (`YYYY-MM-DD`).
- **Timestamps:** ISO 8601 (`YYYY-MM-DDTHH:MM:SSZ`) — **[Zu prüfen]**
  ob mit Zeitzone.
- **Enums:** Als String (Variant-Name).

## Pagination

**[Zu prüfen]** — ob Shifty offset-basierte oder cursor-basierte
Pagination verwendet. In vielen Endpoints scheint es aktuell keine
zu geben (Full-Liste).

## Idempotenz

**[Zu prüfen]** — welche Endpoints sind idempotent (safe retry), welche
nicht.

## Feature-Endpoint-Übersicht

Pro Feature-Cluster gibt es eine eigene REST-Tabelle in der jeweiligen
Feature-Doku:

- [F01 Employee Management](../features/F01-employee-management.md#5-rest-endpoints)
- [F02 Shiftplan Core](../features/F02-shiftplan-core.md#5-rest-endpoints)
- [F03 Booking](../features/F03-booking.md#5-rest-endpoints)
- [F04 Extra Hours](../features/F04-extra-hours.md#5-rest-endpoints)
- [F05 Absence System](../features/F05-absence-system.md#5-rest-endpoints)
- [F06 Vacation Management](../features/F06-vacation-management.md#5-rest-endpoints)
- [F07 Reporting & Balance](../features/F07-reporting-balance.md#5-rest-endpoints)
- [F08 Billing Period](../features/F08-billing-period.md#5-rest-endpoints)
- [F09 Week Metadata](../features/F09-week-metadata.md#5-rest-endpoints)
- [F10 Templates & Communication](../features/F10-templates-communication.md#5-rest-endpoints)
- [F11 Export](../features/F11-export.md#5-rest-endpoints)
- [F12 Auth & Session](../features/F12-auth-session.md#5-rest-endpoints)
- [F13 System Infrastructure](../features/F13-system-infrastructure.md#5-rest-endpoints)

## Für Zweit-Client-Entwickler

Wenn du einen eigenen Client baust (Mobile, CLI, Automation):

1. Lies [`../README.md`](../README.md) und speziell das Prinzip
   "Fat Backend, Thin Client".
2. Generiere Client-Code aus dem OpenAPI-Schema (viele Sprachen haben
   Generator-Tools).
3. Beachte, dass Balance-Rechnung, Konflikt-Prüfung und Snapshot-Semantik
   **nicht** im Client dupliziert werden — immer über die Backend-API.
4. Für Ist-Zustand (Live-Report) einen Endpoint mit hinreichend
   spezifischer Zeitraum-Selektion nutzen.
5. Für Historie (Abrechnung) IMMER Billing-Period-Endpoints — nie
   Live-Rechnung, weil diese driftet.
