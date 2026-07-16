# Shifty — arc42 Architecture Documentation

This is the [arc42](https://arc42.org/) architecture documentation for **Shifty**,
the shift planning and HR management system for small to medium-sized teams.

It documents the **whole system**: the Rust backend (this repository) and the
Dioxus/WASM frontend (`shifty-dioxus/`, developed in-tree but built separately).

## How this relates to the rest of `docs/`

This arc42 documentation is an **overlay, not a replacement**. Shifty already has
detailed reference documentation (see [`docs/README.md`](../README.md)). The arc42
chapters give the *architectural big picture* and link into the existing reference
docs for depth instead of duplicating them. If a linked document and an arc42
chapter ever disagree, the more specific reference document (or the code) wins —
please fix the arc42 chapter.

## Chapters

| # | Chapter | Content |
| --- | --- | --- |
| 1 | [Introduction and Goals](01-introduction-and-goals.md) | What Shifty does, top quality goals, stakeholders |
| 2 | [Architecture Constraints](02-architecture-constraints.md) | Technical, organizational, and convention constraints |
| 3 | [Context and Scope](03-context-and-scope.md) | Business & technical context, external interfaces |
| 4 | [Solution Strategy](04-solution-strategy.md) | Fundamental decisions and solution approaches |
| 5 | [Building Block View](05-building-block-view.md) | Static decomposition: crates, layers, service tiers |
| 6 | [Runtime View](06-runtime-view.md) | Key scenarios: balance report, booking, snapshot, carryover |
| 7 | [Deployment View](07-deployment-view.md) | NixOS deployment, environments, configuration |
| 8 | [Cross-cutting Concepts](08-crosscutting-concepts.md) | Auth, transactions, soft-delete, i18n, time, testing, … |
| 9 | [Architecture Decisions](09-architecture-decisions.md) | ADR-style records of the important decisions |
| 10 | [Quality Requirements](10-quality-requirements.md) | Quality tree and concrete quality scenarios |
| 11 | [Risks and Technical Debt](11-risks-and-technical-debt.md) | Known risks, debt, and documentation drift |
| 12 | [Glossary](12-glossary.md) | Domain and technical terms |

## Conventions used in this documentation

- Diagrams are written in Mermaid so they render directly on GitHub.
- Facts that could not be verified against the code are marked `[To verify]`,
  following the convention of the existing docs.
- Relative links point into the sibling reference documentation
  (`../architecture/`, `../domain/`, `../ops/`, `../api/`, `../features/`).

## Maintenance

Update triggers for this documentation:

- New or removed crate / service tier rule change → chapter 5.
- New external system (IdP, WebDAV target, …) → chapters 3 and 7.
- Significant architectural decision (usually pinned in a GSD phase) → chapter 9.
- New cross-cutting convention → chapter 8.
- Resolved or newly discovered debt → chapter 11.
