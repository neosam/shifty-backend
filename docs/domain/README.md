# Domain Reference — Domain Model & Rules

This section describes **the domain**, not the technology: what a Booking is,
how the time account calculates, when a Billing Period is frozen, what the
difference between Absence and (legacy) Extra Hours is.

If you want to discuss the domain or need to verify a business rule, these
are the documents to read.

## Chapters

- **[glossary.md](./glossary.md)** — Definitions of all domain terms.
- **[time-accounting.md](./time-accounting.md)** — Time account:
  Expected, Worked, Balance, Carryover — how it is calculated and where the
  data comes from.
- **[billing-period.md](./billing-period.md)** — Billing Period snapshot,
  `snapshot_schema_version`, why re-computation must be stable.
- **[absence-system.md](./absence-system.md)** — Range-based absences
  (v1.0+), cutover history, relationship to legacy Extra Hours.
- **[edge-cases.md](./edge-cases.md)** — **Central edge-case reference**.
  Contains time-account edge cases and application-wide edges (auth,
  transactions, time, time zone, rounding, toggle rollouts, …).

## Why a dedicated domain section

Shifty computes non-trivial things. A Booking is simple; a correct Balance
across a contract change, a public holiday on the weekend, a cross-period
sick leave, and a toggle rollout in the middle of the range is not.

This documentation exists so that:

- **Domain reviewers** (non-technical stakeholders) can verify that a rule
  is represented without reading Rust.
- **Backend developers** know, before any change to the Balance
  calculation, which edges must be checked (`edge-cases.md`).
- **Second-client developers** understand what a returned value means —
  without building the calculation themselves.
