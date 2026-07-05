# API-Referenz — REST-Endpoints & Konventionen

Diese Sektion richtet sich an Entwickler:innen, die einen **Zweit-Client**
gegen das Shifty-Backend bauen — Mobile-App, CLI, Data-Export-Tool, etc.

Die Zielsetzung "Fat Backend, Thin Client" garantiert: alle fachlichen Regeln
(Balance-Rechnung, Absence-Konflikte, Billing-Snapshot) leben im Backend und
werden über REST bereitgestellt. Ein neuer Client muss **keine** Domain-Regel
duplizieren.

## Kapitel

- **[openapi.md](./openapi.md)** — Wie du an das OpenAPI-Schema kommst,
  Swagger-UI, Authentifizierung, Error-Mapping.
- **[conventions.md](./conventions.md)** — DTO-Konventionen, Pagination,
  Fehlerformate, Transaction-Semantik über die HTTP-Grenze hinweg,
  Feld-Nullability, Zeit- und Datum-Formate.

## Grundlagen

- **Framework:** Axum (Rust)
- **Doku:** [utoipa](https://docs.rs/utoipa) generiert OpenAPI-Schema aus
  `#[utoipa::path(...)]`-Annotationen und `ToSchema`-Derives auf DTOs.
- **DTOs (Transport Objects, TOs):** Alle DTOs leben im Crate `rest-types`.
  Der Frontend-Client konsumiert dasselbe Crate — es gibt genau eine Quelle
  der Wahrheit für Feldnamen, Typen und Optionalität.
- **Auth:** Entweder OIDC (Prod) oder Mock (`mock_auth`-Feature-Flag).
  Details in [`../architecture/04-auth.md`](../architecture/04-auth.md).
- **Errors:** `ServiceError` wird durch `error_handler` konsistent auf HTTP
  Status Codes gemappt (siehe `conventions.md`).

## Endpoint-Übersicht

Für einen semantischen Überblick, welche Endpoints zu welcher Domäne gehören,
siehe [`../features/`](../features/README.md) — dort ist pro Feature die
zugehörige Endpoint-Liste dokumentiert.

Für die vollständige, maschinenlesbare Referenz: Swagger-UI unter `/swagger-ui`
im laufenden Backend.
