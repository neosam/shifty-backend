# Layer-Architektur

Shifty folgt einer klassischen 3-Schichten-Architektur mit klaren Grenzen.

```
┌───────────────────────────────────────────────┐
│  REST-Layer      rest/         (Axum)         │  HTTP-Handler, DTO-Mapping,
│                                               │  Error → HTTP-Status
├───────────────────────────────────────────────┤
│  Service-Layer   service/, service_impl/      │  Business-Logik, Auth-Gates,
│                                               │  TX-Management, Cross-Domain
├───────────────────────────────────────────────┤
│  DAO-Layer       dao/, dao_impl_sqlite/       │  SQLx-Queries, Row → Entity
│                                               │  Konvertierung
├───────────────────────────────────────────────┤
│  Storage         SQLite (migrations/sqlite/)  │  Schema, Views, Indices
└───────────────────────────────────────────────┘
```

Cross-Cutting-Concerns wie DTOs (`rest-types`) und Utils (`shifty-utils`)
sitzen daneben.

## Warum diese Trennung?

- **REST-Layer** kennt keine Datenbank. Er ist beliebig austauschbar
  (heute Axum, morgen actix-web).
- **Service-Layer** kennt kein HTTP. Er ist über Trait-Interfaces auch
  aus CLI, Job-Runner oder Test-Harness aufrufbar.
- **DAO-Layer** kennt keine Business-Regeln. Er liefert Rows und Entities;
  Auth, Validation und Composition sind Sache der Services.
- **Fat Backend, Thin Client** (siehe Root-README): Sämtliche
  Business-Regeln liegen im Service-Layer. Das Frontend rendert
  Ergebnisse — es rechnet keine Balance selbst.

## Trait-First-Prinzip

Jeder Service und jeder DAO ist zuerst ein **Trait** in `service/` bzw.
`dao/`. Die Implementierung sitzt in `service_impl/` bzw.
`dao_impl_sqlite/`.

Konsequenz:

- **Testbarkeit.** Unit-Tests mocken die Trait-Grenze (via `mockall`),
  ohne Datenbank oder HTTP.
- **Austauschbarkeit.** Ein Postgres-DAO wäre eine parallele Impl neben
  `dao_impl_sqlite/`, ohne Service-Code anzufassen.
- **Explizite Abhängigkeiten.** Die Trait-Deklaration listet
  `type Context`, `type Transaction`, Rückgabe-Fehler-Typ. Nichts ist
  implizit.

## Das `gen_service_impl!`-Makro

Service-Implementierungen werden nicht per Hand als Structs verdrahtet.
Das Makro `gen_service_impl!` (deklariert in
`service_impl/src/macros.rs`) übernimmt:

```rust
gen_service_impl! {
    struct BookingServiceImpl: service::BookingService = BookingServiceDeps {
        BookingDao: dao::BookingDao = booking_dao,
        PermissionService: service::PermissionService = permission_service,
        TransactionDao: dao::TransactionDao = transaction_dao
    }
}
```

Das erzeugt:

- Ein `struct BookingServiceImpl<Deps: BookingServiceDeps>` mit typisierten
  Feldern für jede Dependency.
- Ein Trait `BookingServiceDeps`, das Konsumenten in `main.rs` implementieren,
  damit der DI-Container die konkreten Impls einsetzen kann.
- Konsistente `Arc`- und `Clone`-Behandlung, wo die Async-Grenze das
  verlangt.

## Fehler-Mapping

- **`DaoError`** bei DAO-Aufrufen (SQL-Fehler, DB-Constraints).
- **`ServiceError`** bei Service-Aufrufen; wraps `DaoError`, ergänzt
  Business-Fehler (Forbidden, Conflict, Validation).
- **HTTP-Status** wird im REST-Layer über einen zentralen
  `error_handler`-Wrapper aus `ServiceError` abgeleitet.

Der Effekt: eine einzige Fehler-Konvention pro Layer, keine
Konvertierungs-Explosion.

## Auth-Weitergabe

`Authentication<Context>` ist der Auth-Kontext, den REST-Handler an
Services übergeben. Er wandert durch die gesamte Service-Kette bis zu
den Permission-Checks. Details siehe [`04-auth.md`](./04-auth.md).

## Transaktionen

Jede Service-Methode nimmt `Option<Self::Transaction>` entgegen. Wenn `None`,
öffnet der Service selbst eine Transaktion. Wenn `Some`, fährt er in der
äußeren TX mit. Details siehe [`05-transactions.md`](./05-transactions.md).

## Vertiefung

- [02-service-tiers.md](./02-service-tiers.md) — Basic vs Business-Logic,
  Deps-Regeln.
- [03-data-model.md](./03-data-model.md) — Wie DAOs mit dem Schema
  arbeiten.
- [07-testing.md](./07-testing.md) — Wie das Trait-First-Prinzip
  Tests trägt.
