---
phase: 56-wochen-cron-rollout-backfill-f4
verified: 2026-07-26T08:30:00Z
status: human_needed
score: 5/5 must-haves verified
behavior_unverified: 1
overrides_applied: 0
behavior_unverified_items:
  - truth: "The weekly Monday 03:15 cron fires at runtime and invokes run_once_now on the correct ISO Vorwoche"
    test: "Deploy to a running instance, wait for Monday 03:15, observe logs for 'cron job registered' and 'voluntary-rebooking: booked / skipped' summary"
    expected: "Cron fires without error; for each capped SalesPerson with excess the batch is persisted; for active_from=None all are Skipped{ToggleOff}"
    why_human: "tokio-cron-scheduler job firing cannot be tested by cargo test without a running server and real wall-clock passage; run_once_now delegation is unit-tested but the cron timer activation is not"
human_verification:
  - test: "Verify the weekly cron fires at runtime and processes the Vorwoche correctly"
    expected: "On a Monday early morning the server logs show the cron triggering, listing booked/skipped counts; for a system with active_from=None all SalesPersonen are Skipped{ToggleOff}"
    why_human: "tokio-cron-scheduler timer cannot be observed in cargo test; only the run_once_now function is unit-tested, not the scheduler boot and timer dispatch"
  - test: "Open the Settings page as an admin user and verify Card 5 ('Automatische Freiwilligen-Umbuchung') renders"
    expected: "Card 5 is visible only to admin/HR users; Section 1 shows the Stichtag date input with save/clear; Section 2 shows a from-week input, dry-run and apply buttons; after clicking dry-run a preview table with per-candidate rows appears"
    why_human: "Dioxus RSX rendering and browser-level admin gate can only be verified in a live dx serve or production session"
  - test: "Test the backfill endpoint with a real admin session: POST /admin/rebooking/backfill?from=2026-W01&dry_run=true"
    expected: "Returns 200 with BackfillPreviewTO; no rebooking_batch rows are written (dry_run=true guarantee)"
    why_human: "Live HTTP round-trip with DB requires a running server; the service-layer dry-run guarantee is unit-tested but not exercised end-to-end in this phase"
  - test: "Verify Czech i18n copy in Card 5 with a native Czech speaker"
    expected: "All 21 voluntary rebooking keys in cs.rs read naturally; no awkward machine-translation artifacts"
    why_human: "Czech copy is marked [ASSUMED] by the executor; native-speaker check deferred to manual UAT per established cs.rs convention"
---

# Phase 56: Wochen-Cron + Rollout-Backfill (F4) Verification Report

**Phase Goal:** Ein Wochen-Cron-Job (VoluntaryRebookingScheduler) verarbeitet automatisch die zuletzt abgeschlossene ISO-Woche und schreibt für jeden gedeckelten SalesPerson (cap_planned_hours_to_expected=true) mit Freiwilligen-Überschuss (Ist_vorwoche > Soll_vorwoche + committed_voluntary_vorwoche) die entsprechende Rebooking-Pair-ExtraHours (rebooking_batch kind=auto_cron); ein admin-konfigurierbarer Stichtag-Toggle (voluntary_rebooking_auto_active_from) schützt historische Balance-Views; ein HR-Admin-gated Backfill-REST-Endpoint (POST /admin/rebooking/backfill) erlaubt einmalige rückwirkende Verarbeitung mit Dry-Run-Vorschau.
**Verified:** 2026-07-26
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| 1 | Wochen-Cron (VoluntaryRebookingScheduler) runs Monday 03:15, processes last completed ISO week, writes auto_cron batch for each capped SalesPerson with excess | ⚠️ PRESENT_BEHAVIOR_UNVERIFIED | Scheduler impl exists (service_impl/src/voluntary_rebooking_scheduler.rs), cron `"0 15 3 * * 1"` hardcoded as `VOLUNTARY_REBOOKING_CRON` const, booted in main.rs after pdf_export_scheduler.start(); run_once_now unit-tested (3 tests pass). Cron timer firing cannot be confirmed without runtime. |
| 2 | Stichtag-Toggle (voluntary_rebooking_auto_active_from) gating: active_from=None → all weeks Skipped{ToggleOff}; at-or-after active_from → processed; bytes-identical cargo test baseline | ✓ VERIFIED | voluntary_rebooking_gate.rs: 8 unit tests pass (parse_none/empty/malformed/valid, should_run None/pre/at-or-after, week-53 boundary). Straddling golden-snapshot fixture: 3 scenarios byte-identical (pre/post/straddling active_from). run_once_now_toggle_off_is_full_noop passes. |
| 3 | Cron is idempotent (UNIQUE conflict → Skipped{AlreadyProcessed}); Claim-on-Suggest: pending HR suggestion blocks cron for same (SalesPerson, ISO-week) | ✓ VERIFIED | run_for_week_idempotent_on_conflict passes; run_for_week_respects_pending_suggestion_claim passes. Both via EntityAlreadyExists → AlreadyProcessed path using the Phase-54 UNIQUE index. |
| 4 | POST /admin/rebooking/backfill?from=YYYY-Www&dry_run={true/false} is HR-Admin gated; dry_run=true writes nothing; dry_run=false persists auto_cron_backfill batches; malformed from-week → 400 before service call | ✓ VERIFIED | backfill_range HR_PRIVILEGE gate is first await (service_impl/src/rebooking_reconciliation.rs:676); 7 backfill service tests pass (dry_run writes nothing, apply persists, HR gate, active_from toggle-off, year-boundary); 3 REST OpenAPI contract tests pass (path registered, 200/400/403 declared, DTOs in schema); parse_iso_week unit tests pass (2 tests). |
| 5 | Admin Settings card (Card 5) shows Stichtag editor + backfill form; /admin/rebooking proxy in Dioxus.toml; de/en/cs i18n complete | ✓ VERIFIED | settings.rs has Card 5 with signals/handlers (grep: voluntary_rebooking, backfill, trigger_rebooking_backfill); Dioxus.toml proxy `backend = "http://localhost:3000/admin/rebooking"` present; 21 VoluntaryRebooking Key variants in mod.rs; 21 add_text lines each in de.rs/en.rs/cs.rs; i18n_voluntary_rebooking_keys_present_in_all_locales test passes. |

**Score:** 5/5 truths verified — 4 fully VERIFIED, 1 PRESENT_BEHAVIOR_UNVERIFIED (cron timer firing)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `service_impl/src/voluntary_rebooking_gate.rs` | ISO-week cutoff predicate module | ✓ VERIFIED | Substantive: TOGGLE_NAME const, parse_active_from, should_run_for_week, read_active_from; 8 unit tests; wired in service_impl/src/lib.rs |
| `service_impl/src/test/rebooking_neutrality.rs` | 3 D-06 neutrality property tests | ✓ VERIFIED | test_neutrality_balance, test_neutrality_f1_ist, test_neutrality_f2_soll all pass |
| `service_impl/src/test/billing_period_straddling.rs` | 3 D-01 straddling scenarios | ✓ VERIFIED | test_straddling_snapshot_pre, _post, _straddling all pass; byte-identical |
| `service/src/rebooking_reconciliation.rs` | run_for_week + backfill_range trait methods | ✓ VERIFIED | RunForWeekOutcome, SkipReason, BackfillPreviewItem, run_for_week, backfill_range all present |
| `service_impl/src/rebooking_reconciliation.rs` | run_for_week + backfill_range impl | ✓ VERIFIED | week_excess helper, HR gate at line 676, single write-path via internal rebook() |
| `service_impl/src/test/voluntary_rebooking.rs` | 11 run_for_week tests | ✓ VERIFIED | All 11 tests pass (Booked, NoExcess, AlreadyProcessed, Claim-on-Suggest, Locked-audit, ToggleOff x2, year-boundary W53, week_excess matrix) |
| `service_impl/src/test/rebooking_backfill.rs` | 7 backfill_range tests | ✓ VERIFIED | All 7 pass (dry_run writes nothing, apply persists, HR gate, already-processed, toggle-off, year-boundary, uncapped exclusion) |
| `service/src/voluntary_rebooking_scheduler.rs` | Scheduler trait | ✓ VERIFIED | start() + run_once_now() traits with automock |
| `service_impl/src/voluntary_rebooking_scheduler.rs` | Scheduler impl | ✓ VERIFIED | gen_service_impl! with 5 deps; VOLUNTARY_REBOOKING_CRON = "0 15 3 * * 1"; clone_for_job handle; run_once_now delegates to run_for_week |
| `service_impl/src/test/voluntary_rebooking_scheduler.rs` | 3 scheduler tests | ✓ VERIFIED | run_once_now_books_excess_for_vorwoche, _toggle_off_is_full_noop, _skips_uncapped_person all pass |
| `rest-types/src/rebooking.rs` | BackfillRequestTO, BackfillPreviewTO, BackfillPreviewItemTO | ✓ VERIFIED | 3 TOs with ToSchema; from_domain mapping; skip_reason → lowercase String; serde round-trip tests pass |
| `rest/src/rebooking.rs` | trigger_backfill handler + generate_admin_route + parse_iso_week | ✓ VERIFIED | Handler with #[utoipa::path]; parse_iso_week unit tests pass (2 tests); generate_admin_route registered |
| `rest/src/lib.rs` | /admin/rebooking route + ApiDoc nest | ✓ VERIFIED | Line 753: .nest("/admin/rebooking", rebooking::generate_admin_route()); Line 630: ApiDoc nest entry |
| `rest/tests/rebooking_backfill_test.rs` | 3 OpenAPI surface contract tests | ✓ VERIFIED | backfill_path_registered_under_admin_rebooking, _dtos_registered_as_schemas, _operation_declares_200_400_403 all pass |
| `shifty-dioxus/Dioxus.toml` | /admin/rebooking proxy | ✓ VERIFIED | `backend = "http://localhost:3000/admin/rebooking"` present at line 116 |
| `shifty-dioxus/src/api.rs` | post_rebooking_backfill | ✓ VERIFIED | post_rebooking_backfill function present |
| `shifty-dioxus/src/loader.rs` | toggle getter/setter + trigger_rebooking_backfill | ✓ VERIFIED | VOLUNTARY_REBOOKING_ACTIVE_FROM_TOGGLE const; get/set_voluntary_rebooking_active_from; trigger_rebooking_backfill |
| `shifty-dioxus/src/state/rebooking.rs` | is_valid_iso_week_input + BackfillPreviewItem mapper | ✓ VERIFIED | Pure YYYY-Www gate; From<&BackfillPreviewItemTO> pure field copy; 4 unit tests pass |
| `shifty-dioxus/src/page/settings.rs` | Settings admin card (Card 5) | ✓ VERIFIED | Card present with Stichtag signals/handlers and backfill dry-run/apply wiring |
| `shifty-dioxus/src/i18n/{mod,de,en,cs}.rs` | 21 i18n keys for de/en/cs | ✓ VERIFIED | 21 VoluntaryRebooking Key variants; 21 entries each in de/en/cs; i18n test passes |
| `docs/features/F14-rebooking.md` + `_de.md` | F4 shipped, D-01 decision documented | ✓ VERIFIED | F4 row shows "shipped (Phase 56)"; §9 added; stale "12→13" wording absent (grep count = 0); AutoCronBackfill in both files |
| `docs/features/F07-reporting-balance.md` + `_de.md` | Rebooking-neutral read chains | ✓ VERIFIED | §2.6 documents neutrality filter covers all Rebooking-source writers; D-06 property tests cited |
| `docs/features/F08-billing-period.md` + `_de.md` | CURRENT_SNAPSHOT_SCHEMA_VERSION stays at 12 | ✓ VERIFIED | CURRENT_SNAPSHOT_SCHEMA_VERSION = 12 (service_impl/src/billing_period_report.rs:117); F08 §7.4 records no-bump decision with straddling-fixture proof |
| `docs/architecture/02-service-tiers.md` | VoluntaryRebookingScheduler BL-service listed | ✓ VERIFIED | VoluntaryRebookingScheduler listed with minimal dep set and boot order |
| `docs/architecture/diagrams/service-graph-runtime.mmd` | New scheduler node + edges | ✓ VERIFIED | Node `VoluntaryRebookingScheduler` + edges to RebookingReconciliation and WorkingHours present |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| main.rs | VoluntaryRebookingSchedulerImpl | gen_service_impl! DI construction + .start() | ✓ WIRED | Construction at line 1517; boot at lines 1679-1683 after pdf_export_scheduler.start() |
| VoluntaryRebookingSchedulerImpl::run_once_now | RebookingReconciliationService::run_for_week | run_for_week(sp_id, iso_year, iso_week, AutoCron, Full, None) per capped person | ✓ WIRED | In voluntary_rebooking_scheduler.rs; unit-tested via run_once_now_books_excess_for_vorwoche |
| RebookingReconciliationService::run_for_week | voluntary_rebooking_gate::should_run_for_week | read_active_from + should_run_for_week write-side guard | ✓ WIRED | F4-Self-Guard in service_impl/src/rebooking_reconciliation.rs; ToggleOff tests pass |
| RebookingReconciliationService::run_for_week | internal rebook() pair-write-path | rebook(VolunteerToExtra, excess, ...) | ✓ WIRED | Single write-path at line 653; reused for cron/backfill/manual (Pitfall 15) |
| REST POST /admin/rebooking/backfill | RebookingReconciliationService::backfill_range | error_handler wrapping backfill_range call | ✓ WIRED | trigger_backfill delegates at rest/src/rebooking.rs; HR gate first in service |
| backfill_range | run_for_week | dry_run=false delegates per candidate to run_for_week(AutoCronBackfill) | ✓ WIRED | Single write-path preserved; backfill_apply_persists_autocron_backfill test passes |
| shifty-dioxus settings.rs | loader::trigger_rebooking_backfill | spawn on dry-run/apply button click | ✓ WIRED | Both button click handlers spawn trigger_rebooking_backfill with dry_run=true/false |
| loader.rs | api::post_rebooking_backfill | pass-through to reqwest POST | ✓ WIRED | trigger_rebooking_backfill calls api::post_rebooking_backfill |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Gate unit tests (parse + should_run_for_week) | cargo test -p service_impl voluntary_rebooking_gate | 8 passed | ✓ PASS |
| Neutrality property tests (balance/ist/soll bit-identical) | cargo test -p service_impl rebooking_neutrality | 5 passed | ✓ PASS |
| Straddling snapshot fixture (D-01 byte-identity) | cargo test -p service_impl billing_period_straddling | 3 passed | ✓ PASS |
| run_for_week all cases (Booked/NoExcess/AlreadyProcessed/Locked/ToggleOff) | cargo test -p service_impl voluntary_rebooking | 21 passed | ✓ PASS |
| backfill_range dry_run/apply split | cargo test -p service_impl rebooking_backfill | 7 passed | ✓ PASS |
| Scheduler run_once_now delegation | cargo test -p service_impl voluntary_rebooking_scheduler | included in above 21 | ✓ PASS |
| REST OpenAPI surface contract | cargo test -p rest --test rebooking_backfill_test | 3 passed | ✓ PASS |
| FE rebooking state/i18n unit tests | cargo test --manifest-path shifty-dioxus/Cargo.toml rebooking i18n | 28+59 passed | ✓ PASS |
| Backend clippy | cargo clippy --workspace -- -D warnings | clean | ✓ PASS |
| FE clippy | cargo clippy (shifty-dioxus) -- -D warnings | clean (pre-existing pdf_export_config issue fixed in d63c73aa) | ✓ PASS |

### Probe Execution

No conventional probe scripts declared for this phase. Step 7c: SKIPPED (no probe-*.sh files).

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| REB-AUTO-01 | Plan 02, 03 | Wochen-Cron processes Vorwoche for capped SalesPersons with excess | ✓ SATISFIED | VoluntaryRebookingScheduler + run_for_week engine; 3 scheduler tests + 11 run_for_week tests; booted in main.rs |
| REB-AUTO-02 | Plan 01, 05 | Stichtag-Toggle voluntary_rebooking_auto_active_from; active_from=None = deactivated | ✓ SATISFIED | voluntary_rebooking_gate module; 8 gate tests; FE Stichtag editor; i18n 21 keys; neutrality property-tests CI guard |
| REB-AUTO-03 | Plan 02 | Cron idempotent via UNIQUE index; Claim-on-Suggest | ✓ SATISFIED | AlreadyProcessed via EntityAlreadyExists; idempotent test + claim-on-suggest test both pass |
| REB-AUTO-04 | Plan 02, 04, 05 | POST /admin/rebooking/backfill HR-gated; dry_run=true writes nothing; dry_run=false persists auto_cron_backfill; FE admin card | ✓ SATISFIED | 7 service backfill tests; 3 REST OpenAPI tests; FE Card 5 with dry-run/apply |
| REB-AUTO-05 | Plan 01, 06 | Snapshot version decision documented: stays at 12 with straddling-fixture proof | ✓ SATISFIED | CURRENT_SNAPSHOT_SCHEMA_VERSION = 12 confirmed; 3 straddling tests pass byte-identical; F08/F14 docs updated; stale 12→13 wording removed |

All 5 phase requirements fully covered and satisfied.

### Anti-Patterns Found

No TBD, FIXME, or XXX markers found in any phase-modified files. No placeholder implementations. No stub return values in wired code paths.

| File | Line | Pattern | Severity | Impact |
|------|------|---------|---------|--------|
| (none) | — | — | — | — |

### Verification Focus — D-01, D-02 (Excess Formula) Confirmation

**D-01 (Snapshot version stays at 12):** CONFIRMED. `CURRENT_SNAPSHOT_SCHEMA_VERSION = 12` at `service_impl/src/billing_period_report.rs:117`. Three straddling fixture tests all pass byte-identical. F08 docs record this as resolved. F14 stale "bumps 12→13" wording is absent (grep count = 0).

**D-02 (Excess formula consistency):** CONFIRMED. The formula `week_excess(ist, 0.0, committed)` is internally consistent. `week_voluntary_figures` returns `(ist, soll, soll)` — soll_total IS the committed-voluntary target — so calling `week_excess(ist, 0.0, committed)` computes `ist - (0 + soll_total) = ist - soll_total` (no double-subtraction). The docstring on `week_excess` describing "Ist - (Soll + committed_voluntary)" is accurate for the general function signature but the callers set `soll=0.0` because `soll_total` already IS the committed target. `BackfillPreviewItem.voluntary_soll == committed_voluntary` (same value, documented).

**D-06 (Neutrality chain audit):** CONFIRMED. Three neutrality property-tests (`test_neutrality_balance`, `test_neutrality_f1_ist`, `test_neutrality_f2_soll`) all pass. source=Rebooking rows are filtered at every read-aggregation entry point (Phase-55 D-55-A1 filter). Balance, F1-Ist, and F2-Soll are bit-identical before and after a rebooking pair is written.

**D-03 (Scheduler boot after pdf_export_scheduler):** CONFIRMED. Lines 1671-1683 in main.rs show pdf_export_scheduler.start() called at line 1672 followed by voluntary_rebooking_scheduler.start() at line 1681. 6-field cron `"0 15 3 * * 1"` (Monday 03:15) conflict-free vs pdf_export Monday 06:00.

### Human Verification Required

#### 1. Cron timer fires at runtime on Monday 03:15

**Test:** Deploy to a running instance (or use a clock-manipulation fixture), observe logs on Monday 03:15.
**Expected:** Log line `"voluntary-rebooking: cron job registered ... (Monday 03:15)"` at startup; on Monday early morning, `run_once_now` logs a summary of booked/skipped counts.
**Why human:** tokio-cron-scheduler timer activation requires real wall-clock passage or an integration harness; cargo tests exercise `run_once_now` directly but not the timer dispatch from `start()`.

#### 2. Settings Card 5 renders and backfill form functions in the browser

**Test:** Open Settings as an admin/HR user in `dx serve` or production.
**Expected:** Card 5 "Automatische Freiwilligen-Umbuchung" is visible only for admin/HR. Section 1: Stichtag date input with save/clear saves the `voluntary_rebooking_auto_active_from` toggle. Section 2: from-week input, dry-run button shows preview table, apply button persists rows.
**Why human:** Dioxus RSX rendering, admin role gate, and network round-trip cannot be confirmed by cargo test.

#### 3. Live POST /admin/rebooking/backfill end-to-end

**Test:** With a running backend and an HR-Admin session, POST `/admin/rebooking/backfill?from=2026-W01&dry_run=true`.
**Expected:** 200 response with `BackfillPreviewTO`; no rebooking_batch rows added to DB (confirmed with a follow-up DB query).
**Why human:** Live HTTP + DB state requires a running server; service-layer dry-run guarantee is unit-tested but not exercised as a full stack test in this phase.

#### 4. Czech i18n copy native-speaker review

**Test:** Review the 21 Czech translation strings added to `shifty-dioxus/src/i18n/cs.rs` for voluntary rebooking (keys `SettingsVoluntaryRebooking*`).
**Expected:** All strings read naturally in Czech with correct HR/scheduling terminology.
**Why human:** cs.rs copy is marked `[ASSUMED]` per the established convention for Czech translations; automated i18n key-presence tests only verify non-empty strings exist.

---

## Gaps Summary

No gaps found. All 5 observable truths are either VERIFIED or PRESENT_BEHAVIOR_UNVERIFIED (cron timer firing — code exists and is wired, behavioral test requires runtime). All 5 requirements are satisfied. All modified artifacts are substantive and wired. Backend and FE clippy are clean.

---

_Verified: 2026-07-26_
_Verifier: Claude (gsd-verifier)_
