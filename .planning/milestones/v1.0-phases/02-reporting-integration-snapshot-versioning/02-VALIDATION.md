---
phase: 2
slug: reporting-integration-snapshot-versioning
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-01
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Filled from `02-RESEARCH.md` `## Validation Architecture` — refine in plan-phase.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust workspace) + `mockall` (unit) + in-memory SQLite (integration) |
| **Config file** | none — Cargo workspace |
| **Quick run command** | `cargo test -p service_impl` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~60 s (workspace), ~15 s (service_impl only) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p service_impl --lib`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

> Filled by `gsd-planner` once tasks exist. Below is the **target shape** derived from RESEARCH.md.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 02-01-XX | 01 | 0 | REP-01..04 / SNAP-01..02 | — | Wave 0 fixtures + locking-test scaffold | unit + fixture | `cargo test -p service_impl reporting_phase2_fixtures` | ❌ W0 | ⬜ pending |
| 02-02-XX | 02 | 1 | REP-01 | — | `derive_hours_for_range` returns per-day contract hours, holidays = 0 | unit | `cargo test -p service_impl absence::derive_hours_for_range` | ❌ W0 | ⬜ pending |
| 02-03-XX | 03 | 1 | absence.range_source_active flag plumbing | — | `is_enabled` returns false when row absent | unit | `cargo test -p service_impl toggle::range_source_active` | ❌ W0 | ⬜ pending |
| 02-04-XX | 04 | 2 | REP-02 (bit-identity), REP-03 (switch), SNAP-01/02 | — | flag=off → snapshot bit-identical to v2; flag=on → AbsencePeriods feed report; version pinned to 3 | regression + integration + locking | `cargo test -p service_impl billing_period_report` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `service_impl/src/test/reporting_phase2_fixtures.rs` — shared fixtures (sales person + work_details + holidays + absence_periods + extra_hours twin)
- [ ] `service_impl/src/test/billing_period_snapshot_locking.rs` — locking-test scaffold (exhaustive match over `BillingPeriodValueType` + `assert_eq!(CURRENT_SNAPSHOT_SCHEMA_VERSION, 3)` after bump; pre-bump it asserts 2 and stays red until Wave 2)
- [ ] `service_impl/src/test/absence_derive_hours_range.rs` — placeholder unit test stubs for REP-01 acceptance criteria
- [ ] No new framework install — `cargo test` + `mockall` + in-memory SQLite already in tree (see `service_impl/src/test/`)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| jj-only single-commit atomicity for Wave 2 (Snapshot-Bump + Reporting-Switch) | SNAP-02, REP-03 | jj history is not testable from inside `cargo test` | After Wave 2 commit: `jj log -r @-` must show **one** revision touching `billing_period_report.rs` (`CURRENT_SNAPSHOT_SCHEMA_VERSION = 3`), `reporting.rs` (switch), and the locking test together. |
| OpenAPI schema regen has no surprise diff | (none — quality) | `utoipa` regeneration is a developer-side check | After Wave 2: `cargo run -- --print-openapi > /tmp/openapi.json && diff` against committed schema. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (`reporting_phase2_fixtures`, `billing_period_snapshot_locking`, `absence_derive_hours_range`)
- [ ] No watch-mode flags (`cargo watch` is dev-loop only, never in CI)
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
