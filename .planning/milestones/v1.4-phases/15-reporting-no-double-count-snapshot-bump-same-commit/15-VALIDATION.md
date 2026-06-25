---
phase: 15
slug: reporting-no-double-count-snapshot-bump-same-commit
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-23
---

# Phase 15 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Backend-only calculation phase (Achse B / `booking_information.rs`). No snapshot bump (D-01).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `mockall` for service mocks |
| **Config file** | none — tests in-module (`#[cfg(test)]`) or `service_impl/src/test/` |
| **Quick run command** | `cargo test -p service_impl booking_information` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~60–120 seconds (workspace) |

> NixOS: run inside `nix develop` if cargo/sqlx are not on PATH.

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p service_impl booking_information`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 15-01-xx | 01 | 1 | CVC-04 | — | N/A (internal calc) | unit | `cargo test -p service_impl committed_voluntary` | ❌ W0 | ⬜ pending |
| 15-01-xx | 01 | 1 | CVC-06 | — | cap=false ⇒ 0.0 contribution | unit | `cargo test -p service_impl cvc06` | ❌ W0 | ⬜ pending |
| 15-01-xx | 01 | 1 | CVC-05 | — | snapshot version stays 7 (no bump) | regression | `cargo test -p service_impl billing_period` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Per-Requirement → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| CVC-04 | `max(committed, actual)` per ISO week, summed over year (never `max(Σ,Σ)`) | unit | `cargo test -p service_impl cvc04_sum_not_max_of_sums` | ❌ Wave 0 |
| CVC-04 | `committed=5, actual=7 → counted=7` | unit | `cargo test -p service_impl cvc04_over_fulfilled` | ❌ Wave 0 |
| CVC-04 | `committed=5, actual=3 → counted=5` | unit | `cargo test -p service_impl cvc04_under_fulfilled` | ❌ Wave 0 |
| CVC-04 | Boundary `committed == actual` → committed | unit | `cargo test -p service_impl cvc04_boundary_equal` | ❌ Wave 0 |
| CVC-04 | `committed=5, actual=0 → 5` (forward-looking pledge) | unit | `cargo test -p service_impl cvc04_zero_actual` | ❌ Wave 0 |
| CVC-04 | Empty week (no active work-details rows) → 0.0 | unit | `cargo test -p service_impl cvc04_empty_week` | ❌ Wave 0 |
| CVC-04 | Multi-person aggregation (1 capped+pledge, 1 normal) | unit | `cargo test -p service_impl cvc04_multi_person` | ❌ Wave 0 |
| CVC-06 | `cap=false, committed=5 → contribution 0.0` | unit | `cargo test -p service_impl cvc06_cap_false_zero` | ❌ Wave 0 |
| CVC-06 | `committed=0 → result bit-identical to pre-v1.4` | regression | `cargo test -p service_impl cvc06_committed_zero_backward_compat` | ❌ Wave 0 |
| CVC-05 | `CURRENT_SNAPSHOT_SCHEMA_VERSION` = 7 (unchanged, no bump) | regression | `cargo test -p service_impl billing_period_report` (existing snapshot tests green) | ✅ existing |

> Float comparisons via epsilon (`(a - b).abs() < 0.001`), never `==`.

---

## ✅ Semantic decision RESOLVED — D-05 Two-Band Decomposition (Formula B)

User clarification (2026-06-23, CONTEXT D-05) supersedes the earlier "single max term"
framing AND the RESEARCH "Formula A" recommendation. The model is **two stacked bands
per person/ISO-week**:

- **Band 1 — `committed_voluntary_hours`** (new term/color) = `Σ_week Σ_person committed`
  (flat, cap-gated; via Phase-14 helper).
- **Band 2 — `volunteer_hours`** (EXISTING term, reduced) = `Σ_week Σ_person max(actual_volunteer_p − committed_p, 0)`
  — the surplus ABOVE the pledge. **Per-person subtraction is mandatory** (the `max`
  is nonlinear; persons can overlap). `committed=0 ⇒ no-op ⇒ identical to today.`

No-double-count invariant: per person/week `committed + max(actual − committed, 0) = max(committed, actual)`.

**Multi-person aggregation = FORMULA B = 8.** Worked example (A: cap, c=5, a=0 / B: c=0, a=3):
`committed_voluntary_hours = 5`, `volunteer_hours = 3`, total = **8** (NOT 5).

### Updated/added fixtures from D-05

| Fixture | Person inputs | `committed_voluntary_hours` | `volunteer_hours` (Band 2) | Purpose |
|---------|--------------|-----------------------------|----------------------------|---------|
| `cvc04_multi_person` | A(cap,c=5,a=0) + B(c=0,a=3) | 5.0 | 3.0 (total 8) | Formula B / two-band decomposition |
| `cvc04_band2_surplus` | one is_paid=false person, c=5, a=7 | 5.0 | 2.0 | Band 2 = max(actual−committed,0)=2 |
| `cvc04_band2_pledge_covers` | c=5, a=3 | 5.0 | 0.0 | actual<committed ⇒ Band 2 floored at 0 |
| `cvc06_committed_zero_backward_compat` | c=0, a=7 (is_paid=false) | 0.0 | 7.0 | committed=0 ⇒ volunteer_hours bit-identical to pre-v1.4 |
| `cvc04_paid_capped_band2_zero` | is_paid=true,cap,c=5 (actual_vol=0 in Achse B) | 5.0 | 0.0 | paid person: Band 1 only, Band 2 stays 0 |

---

## Wave 0 Requirements

- [ ] `committed_voluntary_hours: f32` field added to `WeeklySummary` in `service/src/booking_information.rs` — required before tests compile
- [ ] Test module (inline `#[cfg(test)]` in `service_impl/src/booking_information.rs` OR `service_impl/src/test/`) covering all fixtures above
- [ ] No framework install — existing Rust `#[test]` + `mockall` infra covers everything

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| — | — | — | — |

*All phase behaviors have automated verification (pure backend calculation).*

---

## Validation Sign-Off

- [ ] All tasks have automated verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (struct field + test module)
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [x] Multi-person semantic decision resolved → D-05 Two-Band / Formula B; fixtures pinned (committed=5, volunteer=3, total=8)
- [ ] Band 2 (`volunteer_hours`) per-person surplus reduction implemented + tested
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
