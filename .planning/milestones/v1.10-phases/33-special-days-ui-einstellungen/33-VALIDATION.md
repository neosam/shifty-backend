---
phase: 33
slug: special-days-ui-einstellungen
status: ready
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-30
---

# Phase 33 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (backend workspace, mockall) + Dioxus/WASM build gate |
| **Config file** | none — workspace `Cargo.toml` |
| **Quick run command** | `cargo test --workspace special_day` |
| **Full suite command** | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |
| **Frontend gate** | `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown && cargo test` |
| **Estimated runtime** | ~90 seconds (backend) + WASM build |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --workspace special_day`
- **After every plan wave:** Run `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite + WASM build must be green
- **Max feedback latency:** ~90 seconds

---

## Per-Task Verification Map

> Filled by planner/executor as tasks are defined. See 33-RESEARCH.md § "Validation Architecture".

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| {N}-01-01 | 01 | 1 | SPD-{XX} | — | {expected behavior} | unit | `{command}` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `service_impl/src/test/special_days.rs` — new test module (no existing special_days service tests; pattern from `service_impl/src/test/slot.rs`)
- [ ] Frontend api.rs roundtrip test stubs for create/get-year/delete

*Backend test harness (in-memory SQLite, mockall) already exists workspace-wide.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Settings datepicker persist/display loop | SPD-01/02 | WASM datepicker signal behavior (D-25-06) not unit-pinnable | Browser: pick date → create → appears in year list → reload persists |
| Shiftplanner-gating (no 403 mismatch) | SPD-04 | live auth roundtrip | Browser as shiftplanner: create/delete works; as non-shiftplanner: UI hidden |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** ready (plan-time validation passed; wave_0_complete flips true once execute writes the Wave-0 test stubs)
