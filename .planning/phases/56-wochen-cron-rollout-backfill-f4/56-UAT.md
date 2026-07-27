---
status: partial
phase: 56-wochen-cron-rollout-backfill-f4
source: [56-VERIFICATION.md]
started: 2026-07-26
updated: 2026-07-27
---

## Current Test

[testing paused — 3 items outstanding, deferred to the INT environment]

## Tests

### 1. Cron timer fires at runtime on Monday 03:15
expected: Startup log shows the cron registered for Monday 03:15; on trigger, run_once_now logs booked/skipped counts. With active_from=None every SalesPerson is Skipped{ToggleOff} (no regression).
result: pass
evidence: |
  Verified 2026-07-27 against a live `cargo run --bin shifty_bin` (SERVER_ADDRESS=127.0.0.1:3099,
  DATABASE_URL=sqlite:./localdb.sqlite3).

  (a) Registration — startup log, constant unmodified:
      "voluntary-rebooking: cron job registered with schedule '0 15 3 * * 1' (Monday 03:15)"

  (b) Timer dispatch — VOLUNTARY_REBOOKING_CRON temporarily set to "0 * * * * *" (reverted
      afterwards; `git diff` on voluntary_rebooking_scheduler.rs is empty). The job fired on
      consecutive minute boundaries, disproving the v2.3.1 silent-never-fires failure mode:
      19:03:00 / 19:04:00 -> "voluntary-rebooking run finished for KW30/2026: 0 booked, 1 skipped, 0 failed"

  (c) Vorwoche arithmetic — run day 2026-07-27 is ISO 2026-W31; the scheduler processed
      KW30/2026, i.e. the last *completed* week, not the running one.

  (d) Toggle-off regression — toggle `voluntary_rebooking_auto_active_from` has value NULL
      (active_from=None). A seeded SalesPerson with cap_planned_hours_to_expected=1 and a
      2026-W01..W53 contract was counted as `1 skipped`, `0 booked`; rebooking_batch stayed
      at 0 rows. (First run before seeding reported 0/0/0 because the dev DB was empty —
      the loop counts `skipped` only after the cap filter.)

  The seed rows (update_process='uat-56-seed') were deleted again after the run —
  localdb.sqlite3 is back to 0 sales_person / 0 employee_work_details / 0 rebooking_batch,
  the state it was found in.

### 2. Settings Card 5 renders and backfill form functions in the browser
expected: Card 5 "Automatische Freiwilligen-Umbuchung" is visible only for admin/HR. Section 1 (Stichtag date input with save/clear) saves the voluntary_rebooking_auto_active_from toggle. Section 2 (from-week input, dry-run button shows preview table, apply button persists rows).
result: blocked
blocked_by: release-build
reason: "User will test on the INT environment once the release is deployed there, not against a local dx serve."

### 3. Live POST /admin/rebooking/backfill end-to-end
expected: With a running backend and an HR-Admin session, POST /admin/rebooking/backfill?from=2026-W01&dry_run=true returns 200 with a BackfillPreviewTO and writes no rebooking_batch rows (confirm with a follow-up DB query).
result: blocked
blocked_by: release-build
reason: "User will test on the INT environment once the release is deployed there, not against a local backend."

### 4. Czech i18n copy native-speaker review
expected: The 21 Czech voluntary-rebooking strings in shifty-dioxus/src/i18n/cs.rs read naturally with correct HR/scheduling terminology (cs copy is marked [ASSUMED]).
result: blocked
blocked_by: release-build
reason: "Deferred to the INT round together with tests 2-3; additionally needs a native Czech speaker to review the rendered copy."

## Summary

total: 4
passed: 1
issues: 0
pending: 0
skipped: 0
blocked: 3

## Gaps
