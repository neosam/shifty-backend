---
phase: 25
slug: feiertags-auto-anrechnung-stichtag-konfiguration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-28
---

# Phase 25 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | tokio-based unit tests + `mockall` (backend workspace); shifty-dioxus `cargo test` |
| **Config file** | none — workspace-level `cargo test` |
| **Quick run command** | `cargo test holiday_auto_credit` |
| **Full suite command** | `cargo test --workspace` (backend) + `cargo test` (shifty-dioxus) |
| **Clippy gate (hard)** | `cargo clippy --workspace -- -D warnings` |
| **Estimated runtime** | ~60–120 seconds (backend full suite) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test holiday_auto_credit` (+ targeted module test)
- **After every plan wave:** Run `cargo test --workspace` AND `cargo clippy --workspace -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite + clippy must be green; frontend `cargo build --target wasm32-unknown-unknown`
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

> Seeded at requirement level (task IDs assigned by planner). Executor refines per task.

| Req | Behavior | Test Type | Automated Command | File Exists | Status |
|-----|----------|-----------|-------------------|-------------|--------|
| HOL-01 | Auto-credit derives correct hours from `special_day` + contract (`holiday_hours()`, `has_day_of_week`) | unit | `cargo test test_holiday_auto_credit_basic` | ❌ W0 | ⬜ pending |
| HOL-02 | Auto-credit effect == manual `ExtraHours(Holiday)` (holiday_hours + absense_hours + balance) | unit | `cargo test test_holiday_auto_credit_equivalence` | ❌ W0 | ⬜ pending |
| HOL-03 | `booking_information` year-view (`paid_hours`/`committed_voluntary_hours`/`volunteer_hours`) unchanged | unit | `cargo test test_holiday_auto_credit_no_year_view_impact` | ❌ W0 | ⬜ pending |
| HCFG-01 | Holiday before cutoff date → not auto-credited | unit | `cargo test test_holiday_before_cutoff_skipped` | ❌ W0 | ⬜ pending |
| HCFG-02 | Toggle `value` (date) GET/PUT roundtrip persists across reload | manual (browser) + unit (DAO roundtrip) | `cargo test test_toggle_value_roundtrip` | ❌ W0 | ⬜ pending |
| HCFG-03 | Manual `ExtraHours(Holiday)` present → auto-credit skipped (no double-count) | unit | `cargo test test_holiday_manual_wins` | ❌ W0 | ⬜ pending |
| HSNAP-01 | `CURRENT_SNAPSHOT_SCHEMA_VERSION == 11` | unit | `cargo test` (snapshot locking test) | ✅ (update 10→11) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `service_impl/src/test/reporting_holiday_auto_credit.rs` — new module covering HOL-01/02/03, HCFG-01, HCFG-03
- [ ] Register new test module in `service_impl/src/test/mod.rs`
- [ ] Update `billing_period_snapshot_locking.rs` pinned assert: 10 → 11 (HSNAP-01)
- [ ] DAO roundtrip test for toggle `value` get/set (HCFG-02 backend half)

*The reporting/extra-hours/billing-period test infrastructure already exists — only the holiday-auto-credit module is new.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Settings date input set/change/persist-after-reload | HCFG-02 | WASM `<input type=date>` programmatic set does not trigger Dioxus signals (documented caveat) — real user interaction needed | Admin opens `/settings`, sets the holiday "active-from" date, saves, reloads → date persists; non-admin cannot see/edit |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
