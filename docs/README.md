# Shifty — Technical Documentation

Welcome to the technical reference for **Shifty**, the shift planning and
HR management system for small to medium-sized teams.

This documentation is organized by audience. Pick your entry point:

## Entry Points

| If you are … | … then start here |
| --- | --- |
| **new to the project** and want to write code | [`onboarding/`](./onboarding/README.md) |
| going to **run / deploy** the system | [`ops/`](./ops/README.md) |
| building a **custom client** (mobile, CLI, …) against the backend | [`api/`](./api/README.md) |
| trying to understand the **domain rules** (balance, absence, billing) | [`domain/`](./domain/README.md) |
| interested in **how the system is built internally** | [`architecture/`](./architecture/README.md) |
| trying to fully understand a single **feature** | [`features/`](./features/README.md) |

## Structure of This Documentation

```
docs/
├── onboarding/     # Dev onboarding (setup, first week, conventions)
├── ops/            # Operations (Nix deploy, migrations, configuration, release)
├── api/            # REST API reference for second-client developers
├── domain/         # Domain reference (model, balance, absence, billing)
│   └── edge-cases.md  ← Central edge case reference
├── architecture/   # Technical reference (layers, services, DB, auth, tests)
│   └── diagrams/   # Mermaid diagrams (service graph, ER, sequence)
└── features/       # One document per feature domain
```

## Guiding Principles

Shifty follows a few ground rules that you should know before writing code:

1. **Fat Backend, Thin Client.** All business logic lives in the backend. The
   frontend client is a pure view layer. Second-clients (mobile, scripts)
   must not have to duplicate any domain rule.
2. **Everything is a trait.** Services and DAOs are trait definitions with
   swappable implementations. Tests mock at the trait level.
3. **`Option<Transaction>` everywhere.** Every service method can either join
   an existing transaction or open its own. Composite operations run
   atomically.
4. **Soft-delete instead of hard-delete.** All reader queries filter
   `WHERE deleted IS NULL`.
5. **Snapshot-based reporting.** Billing periods are frozen with a
   `snapshot_schema_version`. Later rule changes do not invalidate old
   snapshots.
6. **Clippy is a hard gate.** `nix build` enforces
   `cargo clippy -- --deny warnings`. `cargo test` alone is not enough.

## Freshness of This Documentation

This documentation was produced using the `gsd-docs-update` procedure and is
verified against the codebase. If you find a discrepancy, the code is
authoritative — please flag the document as "stale" and correct it in the
same PR as the code change.

Last full update: see the git log of this directory.
