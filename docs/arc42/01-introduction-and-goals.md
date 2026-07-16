# 1. Introduction and Goals

## 1.1 Requirements Overview

Shifty is a shift planning and HR time-accounting system for small to
medium-sized teams. It answers three questions:

1. **Who works when?** — Shift plans consisting of weekly recurring slots and
   per-week bookings of employees onto those slots, edited by shift planners
   and viewable by everyone.
2. **How many hours does each employee owe / is owed?** — A time account per
   employee: `balance = worked − expected + carryover`, fed by bookings,
   extra hours, range-based absences (vacation, sick leave, unpaid leave, …),
   special days (holidays, short days), and employment contracts.
3. **What was the state at payout time?** — Billing periods freeze
   per-employee metrics as write-once, versioned snapshots so later
   corrections never silently change a past payout.

Secondary features: vacation-day balance, PDF export of shift plans (on-demand
and scheduled push to Nextcloud via WebDAV), personal iCal feeds, text/report
templates, user invitations, admin impersonation, and audit logs.

The functional details are documented per feature cluster in
[`docs/features/`](../features/README.md) (F01–F14) and per domain rule in
[`docs/domain/`](../domain/README.md).

## 1.2 Quality Goals

The three to five qualities that drive the architecture, in priority order:

| # | Quality goal | Motivation and architectural consequence |
| --- | --- | --- |
| 1 | **Correctness of time accounting** | Balances affect salaries and trust. All domain logic lives in the backend ("fat backend, thin client"); calculation rules exist exactly once ([`ReportingService`](../features/F07-reporting-balance.md)); edge cases are centrally documented ([edge-cases.md](../domain/edge-cases.md)); every business rule has unit tests including deny cases. |
| 2 | **Auditability & payout stability** | Past payouts must not change retroactively. Write-once billing snapshots with `snapshot_schema_version`, soft-delete everywhere (no hard deletes), booking log with `created_by`/`deleted_by`, impersonation audit. |
| 3 | **Maintainability by a very small team** | The system must stay evolvable with minimal staffing. Strict layering (REST → Service → DAO), trait-based interfaces with mock-based unit tests, two-tier service rules preventing cyclic dependencies, structured milestone/phase workflow (GSD). |
| 4 | **Reproducibility of builds & deployments** | Deployments must be deterministic and rollback-able. Nix flake builds with clippy `--deny warnings` as a hard gate, SQLx offline query cache, version pinning in the deployment repo (`shifty-nix`). |
| 5 | **Extensibility towards second clients** | Mobile apps, CLIs, or scripts must be able to reuse the API without re-implementing domain rules. Complete REST API with compile-time-generated OpenAPI (utoipa, Swagger UI), shared DTO crate `rest-types`, no domain logic in the frontend. |

## 1.3 Stakeholders

| Role | Concern / expectation |
| --- | --- |
| **Employee** (privilege `sales`) | Sees own shifts ("My Shifts"), own balance and vacation account, manages own absences. Expects correct, understandable numbers. |
| **Shift planner** (privilege `shiftplanner`) | Edits shift plans: bookings, week copy, conflict warnings, week locking. Expects fast editing and non-blocking warnings instead of hard errors. |
| **HR** (privilege `hr`) | Full time-account oversight: reports for all employees, billing periods, absences, entitlements, custom categories. Expects payout-stable snapshots. |
| **Administrator** (privilege `admin`) | User/role/privilege management, impersonation for support, PDF-export configuration, toggles. |
| **Operator / deployer** | Runs the system as a NixOS module with an SQLite file; expects reproducible builds, additive migrations, and a documented backup story ([`docs/ops/`](../ops/README.md)). |
| **Backend developer** | Needs the layering, service-tier, transaction, and testing conventions ([`docs/architecture/`](../architecture/README.md), [`docs/onboarding/`](../onboarding/README.md)). |
| **Second-client developer** | Builds against the REST API only; relies on [`docs/api/`](../api/README.md) and the OpenAPI schema. |
| **Domain reviewer** (non-technical) | Verifies business rules against [`docs/domain/`](../domain/README.md) without reading Rust. |
