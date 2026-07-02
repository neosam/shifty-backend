---
phase: 39
slug: kw-status-grundlage
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-02
---

# Phase 39 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from 39-RESEARCH.md "## Validation Architecture".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust builtin) + `cargo test -p shifty-dioxus` (FE) |
| **Config file** | none — per-crate `Cargo.toml` |
| **Quick run command** | `cargo test -p service_impl -p dao_impl_sqlite week_status` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~60–120 seconds |

---

## Sampling Rate

- **After every task commit:** `cargo test -p service_impl -p dao_impl_sqlite week_status` + `cargo clippy --workspace -- -D warnings`
- **After every plan wave:** `cargo test --workspace` + `cargo build --target wasm32-unknown-unknown` (shifty-dioxus)
- **Before `/gsd-verify-work`:** Full suite green + `cargo sqlx prepare --workspace` committed (.sqlx)
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

| Requirement | Secure Behavior | Test Type | Automated Command | File |
|-------------|-----------------|-----------|-------------------|------|
| WST-01 | KW-53/year-boundary maps via `to_iso_week_date`, never `year()` | pure unit | `cargo test -p service_impl week_status` (iso_week) | `service_impl/src/week_status.rs` |
| WST-01 | Soft-delete ⇔ `Unset` round-trip | integration (in-mem SQLite) | `cargo test -p service_impl week_status` (set_unset_roundtrip) | `service_impl/src/week_status.rs` |
| WST-01 | Only shiftplanner may mutate (permission gate); all roles read | unit + mock | `cargo test -p service_impl week_status` (permission) | `service_impl/src/week_status.rs` |
| WST-01 | All transitions InPlanning↔Planned↔Locked↔Unset | integration | `cargo test -p service_impl week_status` (transitions) | `service_impl/src/week_status.rs` |
| WST-01 | Unknown TEXT discriminant → `DaoError::EnumValueNotFound` | unit (TryFrom) | `cargo test -p dao_impl_sqlite week_status` (unknown_discriminant) | `dao_impl_sqlite/src/week_status.rs` |
| WST-02 | Badge hidden at `Unset` for non-shiftplanner | pure unit (Rust logic) | `cargo test -p shifty-dioxus week_status_badge` | `shifty-dioxus/src/component/week_status_badge.rs` |
| WST-05 | i18n complete: 4 variants × 3 locales (de/en/cs) | unit | `cargo test -p shifty-dioxus i18n` | `shifty-dioxus/src/i18n/` |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `service_impl/src/week_status.rs` — `#[cfg(test)] mod tests` with ISO-week (KW-53) tests, soft-delete/Unset round-trip, permission gate, transitions (in-memory SQLite)
- [ ] `dao_impl_sqlite/src/week_status.rs` — `#[cfg(test)]` for `TryFrom` unknown-discriminant error path
- [ ] `shifty-dioxus/src/component/week_status_badge.rs` — pure-fn visibility-logic unit test
- [ ] `shifty-dioxus/src/i18n/` — i18n completeness test for 4 keys × 3 locales

**Mandatory KW-53 cases** (WST success-criterion 3):
`2021-01-01→(2020,53)`, `2020-12-28→(2020,53)`, `2025-12-29→(2026,1)`, `2025-12-28→(2025,52)`, `2026-03-15→(2026,11)`.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Dropdown live signal round-trip (non-controlled), fresh-fetch after mutation | WST-02 | D-25-06 class — programmatic `<input>`/select signal setting is unreliable in WASM | Optional browser smoke; structural coverage via pure-fn + SSR is the hard gate |

*All other phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have automated verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
