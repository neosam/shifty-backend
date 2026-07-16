# 12. Glossary

Domain terms first, then technical/architectural terms. The domain-normative
source is [`docs/domain/glossary.md`](../domain/glossary.md); this table adds
architecture-specific vocabulary.

## Domain Terms

| Term | Definition |
| --- | --- |
| **Sales Person** | The employee entity (historical name): name, color, `is_paid`, activity bounds, optional link to a User login. Almost every aggregate references it. |
| **User** | A login identity (OIDC or mock) carrying roles and privileges; may be linked 1:1 to a Sales Person. |
| **Contract / Employee Work Details** | A validity-bounded employment segment: expected weekly hours, working-day flags, vacation days, `is_dynamic`, weekly-cap flag. A person can have several segments over time. |
| **Shiftplan** | A named collection of Slots (multiple plans per instance since v2.x). |
| **Slot** | A weekly-recurring time window on a weekday with staffing bounds (`min_resources`, `max_paid_employees`). Defines where work *can* happen. |
| **Booking** | The assignment Sales Person × Slot × (year, calendar week) — the core write object all metrics derive from. |
| **Booking Log** | Read-only audit trail over bookings including soft-deleted rows, with `created_by`/`deleted_by`. |
| **Extra Hours (legacy)** | Single-day time rows (overtime, vacation, sick, holiday, unpaid, volunteer, custom). Still authoritative for pre-cutover history and non-absence categories. |
| **Custom Extra Hours** | Operator-defined extra-hours category with a `modifies_balance` flag. |
| **Absence / Absence Period** | Range-based absence `[from, to]` (inclusive); hours derived at read time from the contract active on each day; supports day fractions. |
| **Cutover** | The v1.0 transition from single-day extra-hours absences to range-based absence periods; both sources coexist for reads. |
| **Expected Hours** | Contractual expectation per week: contract hours minus special days and unpaid leave. |
| **Worked Hours** | Capped shift-plan hours + extra work + balance-modifying custom hours + absence categories that count as worked. |
| **Balance** | `worked − expected + carryover`; the employee's hour surplus (+) or deficit (−). |
| **UnpaidLeave** | Absence category that *reduces expectation only* and adds nothing to the worked side (asymmetric by design). |
| **Carryover** | Persisted year-end balance (hours and vacation days). The row for year *Y* holds the end-of-*Y* value flowing into *Y+1*. |
| **Vacation Balance** | `effective entitlement + carryover(Y−1) − (used + planned)`, in days; used/planned split at "today". |
| **Vacation Entitlement Offset** | Signed whole-day, HR-only correction applied after rounding the contractual entitlement. |
| **Billing Period** | A bounded date range whose per-paid-employee metrics are frozen as a snapshot; periods chain seamlessly; only the latest is deletable. |
| **Snapshot** | The write-once frozen metric rows of a Billing Period, stamped with a `snapshot_schema_version`. |
| **Special Day** | Holiday or short day per (year, week, weekday), affecting expected hours and slot clipping. |
| **Week Status** | Release state of a planning week (in planning / planned / locked); locking gates edits. |
| **Week Message** | Free info text attached to a calendar week. |
| **Block** | Non-persisted read aggregate merging consecutive bookings into contiguous shifts (basis for "My Shifts", iCal, block reports). |
| **Warning** | A derived, non-blocking anomaly (e.g. booking on an absence day) that travels inside success responses and is never persisted. |
| **Rebooking (voluntary)** | Batch mechanism converting capped/voluntary hour surpluses (F14, partially shipped). |

## Technical / Architectural Terms

| Term | Definition |
| --- | --- |
| **TO (Transport Object)** | DTO in `rest-types`, suffix `TO`; the only types on the wire, shared between backend and frontend. |
| **Basic Service** | Service tier managing exactly one aggregate; depends only on DAOs, `PermissionService`, `TransactionDao`, Clock/Uuid. |
| **Business-Logic Service** | Service tier composing basic (and other BL) services acyclically; may use `Full` for internal reads. |
| **`Authentication<Context>`** | Service-layer auth value: `Context(user)` or `Full` (internal all-rights bypass; forbidden in handlers). |
| **`Option<Transaction>`** | Last parameter of every service method: `None` opens an owned transaction, `Some` joins the caller's; ref-counted commit at the outermost owner. |
| **Soft-Delete** | `deleted` timestamp column; readers filter `WHERE deleted IS NULL`; DELETE endpoints never hard-delete. |
| **Re-Point** | Atomically moving bookings between slots in one transaction (double-count protection). |
| **Feature Flag** | Static boolean switch (`feature_flag` table), admin-gated, no user/date context. |
| **Toggle** | User- and/or date-aware switch (`toggle` table) enabling effective-date ("Stichtag") rollouts. |
| **`snapshot_schema_version`** | Version stamp on snapshot rows; bumped whenever formula, value types, or inputs change. |
| **`bookings_view`** | Denormalized SQL view (booking ⋈ person ⋈ slot ⋈ plan) powering read paths like the booking log. |
| **mock_auth / oidc** | Compile-time auth modes: dev auto-admin without IdP vs OpenID Connect against an external IdP. |
| **GSD ("Get Shit Done")** | The active workflow (`.planning/`): milestones → phases with a discuss → plan → execute cycle, pinned design decisions (`D-<phase>-<n>`), requirements with REQ-IDs, and milestone audits; also drives SemVer derivation for releases. |
| **OpenSpec** | Retired spec-driven change workflow in `openspec/` (~2026-03/04); its archive serves as historical decision records. |
| **jj (Jujutsu)** | The VCS used for committing/pushing in this repo (git-compatible). |
| **`.sqlx/` offline cache** | Committed SQLx query metadata enabling compile-time SQL checks without a live DB (`SQLX_OFFLINE=true`). |
| **shifty-nix** | Sibling repository containing the NixOS module, systemd service, static frontend delivery, and the deployed version pin. |
| **dx** | The Dioxus CLI (pinned 0.6.x) used to serve/build the frontend. |
