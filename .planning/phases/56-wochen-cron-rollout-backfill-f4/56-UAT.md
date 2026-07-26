---
status: testing
phase: 56-wochen-cron-rollout-backfill-f4
source: [56-VERIFICATION.md]
started: 2026-07-26
updated: 2026-07-26
---

## Current Test

number: 1
name: Cron timer fires at runtime on Monday 03:15
expected: |
  Log line "voluntary-rebooking: cron job registered ... (Monday 03:15)" at startup;
  on Monday early morning, run_once_now logs a summary of booked/skipped counts. For a
  system with active_from=None all SalesPersonen are Skipped{ToggleOff}.
awaiting: user response

## Tests

### 1. Cron timer fires at runtime on Monday 03:15
expected: Startup log shows the cron registered for Monday 03:15; on trigger, run_once_now logs booked/skipped counts. With active_from=None every SalesPerson is Skipped{ToggleOff} (no regression).
result: [pending]

### 2. Settings Card 5 renders and backfill form functions in the browser
expected: Card 5 "Automatische Freiwilligen-Umbuchung" is visible only for admin/HR. Section 1 (Stichtag date input with save/clear) saves the voluntary_rebooking_auto_active_from toggle. Section 2 (from-week input, dry-run button shows preview table, apply button persists rows).
result: [pending]

### 3. Live POST /admin/rebooking/backfill end-to-end
expected: With a running backend and an HR-Admin session, POST /admin/rebooking/backfill?from=2026-W01&dry_run=true returns 200 with a BackfillPreviewTO and writes no rebooking_batch rows (confirm with a follow-up DB query).
result: [pending]

### 4. Czech i18n copy native-speaker review
expected: The 21 Czech voluntary-rebooking strings in shifty-dioxus/src/i18n/cs.rs read naturally with correct HR/scheduling terminology (cs copy is marked [ASSUMED]).
result: [pending]

## Summary

total: 4
passed: 0
issues: 0
pending: 4
skipped: 0
blocked: 0

## Gaps
