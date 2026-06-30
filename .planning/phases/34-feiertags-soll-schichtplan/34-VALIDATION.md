---
phase: 34
slug: feiertags-soll-schichtplan
status: ready
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-30
---

# Phase 34 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution. BE-only phase.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (service_impl, mockall) |
| **Config file** | none — workspace `Cargo.toml` |
| **Quick run command** | `cargo test -p service_impl reporting_holiday` |
| **Full suite command** | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |
| **Estimated runtime** | ~90 seconds |

---

## Sampling Rate

- **After every task commit:** `cargo test -p service_impl reporting_holiday`
- **After every plan wave:** `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Before `/gsd-verify-work`:** full suite green
- **Max feedback latency:** ~90 seconds

---

## Per-Task Verification Map

> See 34-RESEARCH.md § "Validation Architecture". The core invariants:

| Truth | Requirement | Test | Type |
|-------|-------------|------|------|
| derived holiday reduces `expected_hours` (40→32) | HSP-01 | rebuilt `test_holiday_auto_credit_no_year_view_impact` | unit |
| `holiday_hours` filled (0→8) | HSP-02 | same test, positive assert | unit |
| `dynamic_hours` band UNCHANGED (==40) | HSP-03 | same test, band-guard assert | unit |
| holiday before Stichtag → no effect (expected==40, holiday==0) | HSP-04 | new subtest | unit |
| manual ExtraHours(Holiday) wins → no double-count | HSP-04 | new subtest | unit |
| no snapshot bump (billing_period reads not from get_week) | D-34-04 | grep/read verification task | manual-grep |

*Status: ⬜ pending — filled by executor.*

---

## Wave 0 Requirements

- [ ] Rebuild `service_impl/src/test/reporting_holiday_auto_credit.rs:545` test module (mock expectations for special_day + toggle)
- [ ] 2 new HSP-04 subtests (before-cutoff gate; manual-wins no-double-count)

*Existing reporting test harness (mockall, in-memory) covers the fixtures.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Snapshot version stays 12 | D-34-04 | grep evidence, not a unit test | grep `billing_period_report.rs` for get_week/booking_information refs → none expected; CURRENT_SNAPSHOT_SCHEMA_VERSION unchanged |

*BE-only phase: no browser/UI verification (D-34-02).*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Band invariance (dynamic_hours==40) asserted before+after
- [ ] Stichtag gating + manual-wins covered
- [ ] No watch-mode flags
- [ ] `nyquist_compliant: true`

**Approval:** ready (plan-time)
