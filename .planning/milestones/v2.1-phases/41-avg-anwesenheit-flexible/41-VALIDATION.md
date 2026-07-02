---
phase: 41
slug: avg-anwesenheit-flexible
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-02
---

# Phase 41 — Validation Strategy

> Per-phase validation contract. Derived from 41-RESEARCH.md "## Validation Architecture".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` (backend); `cargo test -p shifty-dioxus` (FE) |
| **Config file** | none — workspace default |
| **Quick run command** | `cargo test -p service_impl reporting_avg_attendance` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~60–120 seconds |

---

## Sampling Rate

- **After every task commit:** `cargo test -p service_impl reporting_avg_attendance` + `cargo clippy --workspace -- -D warnings`
- **After every plan wave:** `cargo test --workspace` (+ FE: `cargo build --target wasm32-unknown-unknown` + `cargo test -p shifty-dioxus`)
- **Before `/gsd-verify-work`:** Full suite green + mandatory grep `CURRENT_SNAPSHOT_SCHEMA_VERSION.*12` confirmed
- **Max feedback latency:** ~120 seconds

---

## Requirements → Test Map

| Req | Behavior | Test type | Command | Wave 0? |
|-----|----------|-----------|---------|---------|
| AVG-01 | 12 attendance days, 54h → 4.5 h/day (user example) | pure-fn unit | `cargo test -p service_impl reporting_avg_attendance::user_example` | new |
| AVG-01 | Absence day (Vacation=8h, no work) → not counted | pure-fn unit | `reporting_avg_attendance::absence_day_not_counted` | new |
| AVG-01 | Mixed day (Shiftplan 4h + Vacation 4h) → attendance day, numerator = 4h | pure-fn unit | `reporting_avg_attendance::mixed_day_counts_work_only` | new |
| AVG-01 | Custom category → not an attendance day | pure-fn unit | `reporting_avg_attendance::custom_category_not_attendance` | new |
| AVG-01 | Empty slice → attendance_days=0, avg=None | pure-fn unit | `reporting_avg_attendance::empty_slice_returns_none` | new |
| AVG-02/06 | 1 attendance day → avg=None (< 2 threshold) | pure-fn unit | `reporting_avg_attendance::one_day_returns_none` | new |
| AVG-02/06 | 2 attendance days → avg=Some(f32) | pure-fn unit | `reporting_avg_attendance::two_days_returns_some` | new |
| AVG-01 | A-22-1 (average_worked_hours_per_week) UNCHANGED | regression unit | `cargo test -p service_impl reporting_avg_weekly` | exists |
| AVG-01 | Snapshot version stays 12 | constant assertion | `grep "CURRENT_SNAPSHOT_SCHEMA_VERSION.*12" service_impl/src/billing_period_report.rs` | grep |
| AVG-02 | HR gate: non-HR user → Forbidden | mock unit | `reporting::attendance_statistics_requires_hr` | new |
| AVG-02 | is_dynamic filter: non-flexible employee → None | mock unit | `reporting::attendance_statistics_returns_none_for_static` | new |
| AVG-03 | i18n keys present in de/en/cs | i18n unit | `cargo test -p shifty-dioxus i18n_attendance_keys_present_in_all_locales` | new |

*Status: ⬜ pending · ✅ green · ❌ red*

---

## Wave 0 Requirements

- [ ] `service_impl/src/test/reporting_avg_attendance.rs` — all AVG-01/06 pure-fn cases (analog `reporting_avg_weekly.rs`)
- [ ] Mock tests for HR-gate + is_dynamic filter in `service_impl/src/reporting.rs` `#[cfg(test)]`
- [ ] `i18n_attendance_keys_present_in_all_locales` in `shifty-dioxus` (analog `i18n_week_status_keys_present_in_all_locales`)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| The new number renders next to "Ø Std/Woche" in the HR stats block for a flexible employee; "–" empty state at <2 days; absent for non-flexible | AVG-02 | WASM report page rendering (get_page_text/find works, screenshots time out — reference note) | Optional smoke; the pure-fn + endpoint tests are the hard gate |

*All computation + gating behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] Pure-fn covers user example, absence exclusion, mixed day, threshold <2, empty
- [ ] HR-gate + is_dynamic filter tested
- [ ] A-22-1 regression test still green (unchanged)
- [ ] Snapshot version 12 grep-asserted (no bump)
- [ ] i18n de/en/cs completeness test
- [ ] No watch-mode flags; feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
