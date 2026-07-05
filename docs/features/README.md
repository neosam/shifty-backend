# Features — One Document per Domain

This section is the **feature reference**. Each file fully describes a
feature cluster of Shifty: business context, technical details, edge
cases, and test coverage.

## Structure of a Feature Doc

Every feature document follows the same outline:

1. **What is it?** — Short business description, target user group in the UI.
2. **Business rules** — All business constraints spelled out.
3. **Data model** — Tables, columns, relationships, relevant migrations.
4. **Service API** — Trait methods, auth gates, TX behavior.
5. **REST endpoints** — Paths, methods, DTOs, error cases.
6. **Frontend integration** — Which pages / components use the feature.
7. **Edge cases** — References to `../domain/edge-cases.md` + feature-specific ones.
8. **Tests** — Where unit/integration coverage lives, what is NOT covered.
9. **History** — Milestone context, why the feature looks like this (Cutover,
   toggle rollout, etc.).

## Feature Clusters

| # | Cluster | File |
| --- | --- | --- |
| F01 | Employee Management (Sales Person, contract, unavailability) | [F01-employee-management.md](./F01-employee-management.md) |
| F02 | Shiftplan Core (Slots, catalog, editor, view) | [F02-shiftplan-core.md](./F02-shiftplan-core.md) |
| F03 | Booking (assignment, log, information) | [F03-booking.md](./F03-booking.md) |
| F04 | Extra Hours — legacy time recording + custom categories | [F04-extra-hours.md](./F04-extra-hours.md) |
| F05 | Absence System (range-based, v1.0+) | [F05-absence-system.md](./F05-absence-system.md) |
| F06 | Vacation Management (Balance, offset, Carryover) | [F06-vacation-management.md](./F06-vacation-management.md) |
| F07 | Reporting & Balance calculation | [F07-reporting-balance.md](./F07-reporting-balance.md) |
| F08 | Billing Period (Snapshot + versioning) | [F08-billing-period.md](./F08-billing-period.md) |
| F09 | Special Days, week status, week message, warning | [F09-week-metadata.md](./F09-week-metadata.md) |
| F10 | Templates & communication (text templates, user invitation) | [F10-templates-communication.md](./F10-templates-communication.md) |
| F11 | Export (PDF shiftplan, iCal, WebDAV, scheduler) | [F11-export.md](./F11-export.md) |
| F12 | Auth & session (OIDC, mock, impersonation, permissions) | [F12-auth-session.md](./F12-auth-session.md) |
| F13 | System infrastructure (Feature Flags, Toggles, scheduler, clock, UUID) | [F13-system-infrastructure.md](./F13-system-infrastructure.md) |

## Relation to Existing Docs

Some features already have older, specialized docs in the `docs/`
directory:

- `absence-feature-frontend.md` → referenced in `F05-absence-system.md`.
- `employee-management.md` / `_de.md` → referenced and supplemented in
  `F01-employee-management.md`.
- `block-report-templates/`, `template-examples/`, `test-examples/` →
  referenced from `F07-reporting-balance.md` and
  `F10-templates-communication.md`, respectively.

The new feature documents are **the authoritative reference**. The older
documents will be pulled in incrementally.
