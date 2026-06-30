---
phase: 35
slug: slot-einzelwoche-aenderung
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-30
---

# Phase 35 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust, mockall unit tests + in-memory SQLite integration) |
| **Config file** | none — workspace `Cargo.toml`; `SQLX_OFFLINE=true` + `.sqlx` cache |
| **Quick run command** | `SQLX_OFFLINE=true cargo test -p service_impl shiftplan_edit` |
| **Full suite command** | `SQLX_OFFLINE=true cargo test --workspace` |
| **Estimated runtime** | ~90 seconds (full workspace) |

> Hard gate (project): `cargo clippy --workspace -- -D warnings` must also be clean before commit.

---

## Sampling Rate

- **After every task commit:** Run `SQLX_OFFLINE=true cargo test -p service_impl shiftplan_edit`
- **After every plan wave:** Run `SQLX_OFFLINE=true cargo test --workspace` + `cargo clippy --workspace -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite + clippy green
- **Max feedback latency:** ~90 seconds

---

## Per-Task Verification Map

> Filled by the planner — each behavior-adding task gets an automated `cargo test` verify.
> Critical SWO-04 coverage (D-35-05): booking re-point split (Segment 2 vs 3, exactly once),
> no double-count in reports/balance (`deleted IS NULL`), rollback-on-error, edge cases
> (exception KW == first KW; slot with `valid_to = None`; exception KW without bookings).

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 35-01-01 | 01 | 1 | SWO-02/04 | — | 3-segment split + booking partition, no double-count | unit | `cargo test -p service_impl shiftplan_edit` | ✅ | ⬜ pending |

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.* `service_impl/src/test/shiftplan_edit.rs` is the established harness (mockall + in-memory) — extend it for D-35-05.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Editor mode toggle ("nur diese Woche" / "ab dieser Woche") visible + wired | SWO-01 | Dioxus WASM UI interaction not reliably automatable (D-25-06 caveat) | Browser smoke: open slot editor, pick "nur diese Woche", save, confirm only the chosen KW changed |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
