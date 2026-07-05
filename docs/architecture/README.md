# Architecture Reference — What Shifty Looks Like Inside

This section is the **technical** reference. Target audience: anyone working
on the code or wanting to understand why it is structured the way it is.

## Chapters

- **[01-layered.md](./01-layered.md)** — Layered architecture: REST → Service →
  DAO → SQLite. The `gen_service_impl!` macro. Error mapping.
- **[02-service-tiers.md](./02-service-tiers.md)** — Basic Services vs
  Business-Logic Services. Dependency rules, DI ordering. Includes the
  service dependency graph.
- **[03-data-model.md](./03-data-model.md)** — DB schema (ER), logical
  domain model (aggregates), soft-delete convention.
- **[04-auth.md](./04-auth.md)** — `mock_auth` vs OIDC, RBAC roles,
  `Authentication<Context>`, the `Full` bypass for internal aggregates.
- **[05-transactions.md](./05-transactions.md)** — `Option<Transaction>`,
  atomic re-points, rollback semantics.
- **[06-frontend.md](./06-frontend.md)** — Dioxus/WASM architecture, dx-CLI
  0.6.x pin, proxy configuration.
- **[07-testing.md](./07-testing.md)** — Mockall unit tests,
  in-memory SQLite integration tests, `cargo sqlx prepare`,
  clippy gate, toolchain split.
- **[08-i18n.md](./08-i18n.md)** — Three-language convention (En/De/Cs), how
  new strings get introduced.

## Diagrams

All diagrams are Mermaid files, rendered live in GitHub and `mdBook`.

- `diagrams/service-graph-runtime.mmd` — What `main.rs` actually wires up.
- `diagrams/service-graph-traits.mmd` — What the trait deps declare.
- `diagrams/db-schema-er.mmd` — Physical ER model derived from `migrations/`.
- `diagrams/domain-aggregates.mmd` — Logical model with aggregate
  boundaries.
- `diagrams/sequence-booking-create.mmd` — Request flow for booking creation.
- `diagrams/sequence-report.mmd` — Reporting flow including carryover lookup.

## Reference Sources

The architecture follows conventions from:

- Root `CLAUDE.md` — project conventions (short form).
- `shifty-backend/CLAUDE.md` — backend specifics, service-tier rules.
- `.planning/` — historical ADRs and phase decisions.

This documentation is the long-form, lookup-friendly counterpart.
