# Absence System (Domain)

This file explains the range-based absence system from the domain
perspective. For the technical reference see
[F05 Absence System](../features/F05-absence-system.md).

## What is an Absence?

An **Absence** is an absence period of a Sales Person with a **start**,
an **end**, and a **category** (Vacation, SickLeave, UnpaidLeave,
VolunteerWork, …).

**Range semantics:** inclusive on both sides — `[from, to]`. A range
`from=2026-06-01`, `to=2026-06-01` is "exactly one day of vacation".

## Why ranges instead of single days?

Before v1.0 absences were maintained as **Extra Hours rows per day**.
That led to:

- **Data explosion:** Two weeks of vacation = 14 rows.
- **Change overhead:** Extending vacation by one day = inserting a new
  row and refreshing aggregates.
- **Loss of semantics:** "These 14 rows belong together" was only in the
  user's head.

The Absence system turns this into **a single row** covering 14 days.

## Categories

Absences use the same categories as Extra Hours (see
[`glossary.md`](./glossary.md)):

- **Vacation** — Vacation. Counts as "worked" for the Balance.
- **SickLeave** — Sick leave. Counts as "worked" for the Balance.
- **UnpaidLeave** — Unpaid leave. Reduces the expectation, adds nothing.
- **VolunteerWork** — Volunteer work. Counts as "worked".
- **Unavailable** — Availability block.
- **Holiday** — Public holiday as an individual Absence (rare; usually
  handled via Special Days).

The semantics for the Balance calculation are identical to Extra Hours
— reporting aggregates both sources.

## Cutover history

The transition from Extra-Hours-based single-day rows to Absence ranges
is called the **cutover** and happened in Milestone v1.0.

Before cutover: All absences in `extra_hours`.
From cutover onwards: New absences in `absence_period`. Old ones remain
in `extra_hours` and are not migrated (exception: `absence_conversion`
for explicit conversion).

**Consequence for reporting:** All reader paths that aggregate
Absence-relevant categories (Vacation, SickLeave, UnpaidLeave,
VolunteerWork) MUST read from **both** sources and merge them —
otherwise historical rows are missing.

## Conflict semantics

### Absence-vs-Absence (Overlap)

- **Same-category overlap:** Forbidden — e.g. two overlapping Vacation
  ranges for the same Sales Person. The service rejects the insert.
- **Cross-category overlap:** Allowed with priority —
  `SickLeave > Vacation > UnpaidLeave`. That means: if Vacation and
  SickLeave overlap, the overlapping range is counted as SickLeave.

### Absence-vs-Booking

Non-blocking. There is a **warning** ("There is a Booking during the
Absence"), but the insert is not rejected. The user can decide whether
the Booking should be deleted.

Domain motivation: In practice, Bookings are often planned in advance,
and a spontaneous sick day should not prevent the Absence from being
created.

### Absence over a non-working day

A Vacation range that includes a Sunday which the Sales Person does not
work per contract counts as **0 hours** for the Sunday. The range is
still valid.

## Auth model

- **HR** can create, modify, delete Absences for everyone.
- **Sales Person themselves** can create, modify, delete their own
  Absences (with certain restrictions; **[To verify]** exact rules in
  F05).
- **`find_all`** is HR-only — Sales-Person-owned reads are filtered to
  their own rows.

## Conversion: Legacy → Absence

The `AbsenceConversionService` (`service_impl/src/absence_conversion.rs`)
is the way to actively transfer legacy Extra Hours rows into Absence
ranges. Only a one-time data move — not part of the live reporting
path.

## Toggle rollout chain

The feature rollout D-51-07/HCFG-02 uses toggles with an effective date:
before the effective date the old semantics apply (only Extra Hours),
after the effective date the new semantics apply (Absence system
active).

**[Convention]** (from memory): Per consumer chain, the "toggle off"
branch reconstructs the old semantics — do not blindly assume "None →
raw".

## Balance calculation with Absence

For a time range + Sales Person the reporting counts:

1. All Bookings (always).
2. All Extra Hours rows (always, including after cutover for legacy
   data).
3. All Absences (after cutover).

Categories are treated with their standard semantics (see
[`time-accounting.md`](./time-accounting.md)):

- Vacation/SickLeave/VolunteerWork/Unavailable/Holiday: add to the
  actual side.
- UnpaidLeave: reduces expectation.

## Edge-case references

See [`edge-cases.md#2-absence--extra-hours`](./edge-cases.md#2-absence--extra-hours):

- Range across a Billing Period boundary (split?).
- Range across the year boundary (Carryover interaction).
- Cross-category overlap (priority).
- Absence on a non-working day.
- Absence vs Booking conflict (warning, not block).

## PR review pattern

**On changes to `reporting.rs` or `absence.rs`:**

1. Are **both** sources (`extra_hours` + `absence_period`) read for
   Absence-relevant categories?
2. Is the toggle rollout's "toggle off" branch maintained for the
   affected consumer chain?
3. Are there tests for year-boundary ranges and Carryover interaction?

Without these checks, cutover consistency drifts silently.
