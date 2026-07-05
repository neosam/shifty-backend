# Architektur-Referenz — Wie Shifty innen aussieht

Diese Sektion ist die **technische** Referenz. Zielgruppe: alle, die am Code
arbeiten oder verstehen wollen, warum er so aufgebaut ist.

## Kapitel

- **[01-layered.md](./01-layered.md)** — Layer-Architektur: REST → Service →
  DAO → SQLite. Das `gen_service_impl!`-Makro. Fehler-Mapping.
- **[02-service-tiers.md](./02-service-tiers.md)** — Basic-Services vs
  Business-Logic-Services. Deps-Regeln, DI-Reihenfolge. Enthält den
  Service-Dependency-Graph.
- **[03-data-model.md](./03-data-model.md)** — DB-Schema (ER), logisches
  Domain-Modell (Aggregate), Soft-Delete-Konvention.
- **[04-auth.md](./04-auth.md)** — `mock_auth` vs OIDC, RBAC-Rollen,
  `Authentication<Context>`, der `Full`-Bypass für interne Aggregate.
- **[05-transactions.md](./05-transactions.md)** — `Option<Transaction>`,
  atomare Re-Points, Rollback-Semantik.
- **[06-frontend.md](./06-frontend.md)** — Dioxus/WASM-Architektur, dx-CLI
  0.6.x-Pin, Proxy-Konfiguration.
- **[07-testing.md](./07-testing.md)** — Mockall-Unit-Tests,
  In-Mem-SQLite-Integrationstests, `cargo sqlx prepare`,
  Clippy-Gate, Toolchain-Split.
- **[08-i18n.md](./08-i18n.md)** — Drei-Sprachen-Konvention (En/De/Cs), wie
  neue Strings eingeführt werden.

## Diagramme

Alle Diagramme sind Mermaid-Dateien, live-rendert in GitHub und `mdBook`.

- `diagrams/service-graph-runtime.mmd` — Was `main.rs` tatsächlich verdrahtet.
- `diagrams/service-graph-traits.mmd` — Was die Trait-Deps deklarieren.
- `diagrams/db-schema-er.mmd` — Physisches ER-Modell aus `migrations/`.
- `diagrams/domain-aggregates.mmd` — Logisches Modell mit
  Aggregate-Boundaries.
- `diagrams/sequence-booking-create.mmd` — Request-Fluss beim Booking-Anlegen.
- `diagrams/sequence-report.mmd` — Reporting-Fluss inkl. Carryover-Lookup.

## Referenz-Quellen

Die Architektur folgt Konventionen aus:

- Root-`CLAUDE.md` — Projekt-Konventionen (Kurzform).
- `shifty-backend/CLAUDE.md` — Backend-Spezifika, Service-Tier-Regeln.
- `.planning/` — Historische ADRs und Phase-Entscheidungen.

Diese Doku ist das nachschlagbare Langformat.
