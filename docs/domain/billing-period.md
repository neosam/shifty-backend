# Billing Period — Snapshot & Versioning (Domain)

This file explains the Billing Period concept from the domain perspective.
For the technical reference see
[F08 Billing Period](../features/F08-billing-period.md).

## What is a Billing Period?

A **Billing Period** is a bounded billing time range for which, at a
specific point in time, a **Snapshot** of every Sales Person's Balance,
hours, and vacation figures is generated and frozen in the database.

**Purpose:** A billing document / printout / payout must not depend on
exactly when the report is opened. If the HR person generated the
Snapshot for the previous month on the 3rd, it must display the same
value on the 30th of the same month — even if Bookings were corrected
retroactively in the meantime.

## Contract: Write-Once + Versioned

Two rules make the Snapshot reliable:

### Rule 1: Write-Once

Once a Billing Period is written, its contents are fixed. Subsequent
Bookings, Absences, Extra Hours are **not** written back into the
Snapshot. This is intentional.

**Consequence for the user:** The live report (Weekly Overview, My
Shifts) may deviate from the Billing Period if data was changed after
the fact. Both numbers are valid — one is "what it was on the 3rd", the
other is "what it is today".

### Rule 2: Versioning (`snapshot_schema_version`)

Every Billing Period row carries a version number:
`snapshot_schema_version: u32`, currently **12**.

This number says: "At the time this Snapshot was written, these
calculation rules were in effect."

**If the calculation rules change** (new category, different formula,
different input set), the version must be incremented by 1. The reason:
A validator that compares the live calculation against the Snapshot
would otherwise be unable to distinguish between:

- "The Snapshot is wrong" (real data bug) and
- "The Snapshot was written under the old rules" (expected diff due to
  a rule change).

Without the version, all old Snapshots would suddenly be "wrong" after a
formula change — any post-mortem would be pointless.

**With the version:** The validator reads the Snapshot's version. If
lower than the current constant → old rules, diff expected. If equal →
current rules, diff is a bug.

## When a bump is required

**Bump the version when you:**

1. Add a new persisted `value_type` to `billing_period_sales_person`.
2. Remove or rename an existing `value_type`.
3. Change the computation of an existing `value_type`.
4. Change the input set (e.g. co-aggregate a new Extra Hours category).

**Do NOT bump when you:**

1. Make purely additive changes that do not touch any `value_type` (new
   REST endpoints, frontend changes, new columns on unrelated tables).
2. Do internal refactorings that produce identical output.

**Technical reference:** The constant is
`service_impl::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION`
and is stamped by the writer in
`build_and_persist_billing_period_report()`. See
[F08](../features/F08-billing-period.md) for the code reference.

## Four report views per Snapshot

A Snapshot contains four aggregate views:

- **`value_ytd_from`** — Year-to-date up to the start of the period.
- **`value_ytd_to`** — Year-to-date up to the end of the period.
- **`value_full_year`** — Full-year sum.
- **`value_delta`** — Difference end − start (= what happened during the
  period).

For every `value_type` (Balance, Worked Hours, Vacation Used, …) these
four views exist.

## Who is allowed to read which Snapshot

- **HR:** See and create all Billing Periods.
- **Sales Person:** Only their own history in Billing Period Details.
- **[To verify]** exact auth gates — see F08.

## When Snapshots come into play

- **End of month / end of quarter:** HR generates the Snapshot for the
  past period. From that moment on the number is "official".
- **End of year:** The full-year Snapshot serves as the basis for the
  Carryover into the following year.
- **Ad-hoc:** HR can generate a time-range Snapshot at any time — for
  reference letters, billing statements, audits.

## What is NOT in the Snapshot

- **Booking details** — only the aggregated hour is persisted, not a
  row-by-row view.
- **Absence details** — only the sums.
- **Textual comments** — warnings, Week Messages are live-only.

## Edge-case references

See [`edge-cases.md#3-billing-period--snapshots`](./edge-cases.md#3-billing-period--snapshots)
for the sharp edges:

- Old Snapshot on new code.
- Forgotten bump after a formula change.
- Race Snapshot ↔ parallel write.
- No Snapshot present — live calculation falls back in.
- Toggle-based semantic change — MUST bump.

## PR review pattern

**Mandatory check for changes to `billing_period_report.rs`:**

1. Was `CURRENT_SNAPSHOT_SCHEMA_VERSION` bumped?
2. If yes, is the PR text documenting why?
3. If no, has it been clarified that the change is truly additive?

Without this check, Snapshot semantics drift silently.
