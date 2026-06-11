---
phase: 9
slug: booking-flow-reverse-warnings-copy-week
status: planned
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-11
---

# Phase 9 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust unit/SSR-snapshot tests in `shifty-dioxus/`) |
| **Config file** | none — existing test infra in `shifty-dioxus/` |
| **Quick run command** | `cargo test --package shifty-dioxus` (from `shifty-dioxus/`) |
| **Full suite command** | `cargo test --package shifty-dioxus` + `cargo build --target wasm32-unknown-unknown` (WASM-Build-Gate) |
| **Estimated runtime** | ~60-120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --package shifty-dioxus` (from `shifty-dioxus/`)
- **After every plan wave:** Run `cargo test --package shifty-dioxus` + `cargo build --target wasm32-unknown-unknown`
- **Before `/gsd-verify-work`:** Full suite green + WASM build exit 0 + backend `cargo check --workspace`
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 9-01-01 | 01 | 1 | FUI-A-05 | T-09-02 | escaped text render | i18n parity + reference | `cargo test i18n_booking_warning` | ❌ W0 (creates) | ⬜ pending |
| 9-01-02 | 01 | 1 | FUI-A-05 | T-09-01/02 | typed payload, escaped render | SSR snapshot | `cargo test --package shifty-dioxus warning_list` | ❌ W0 (creates) | ⬜ pending |
| 9-01-03 | 01 | 1 | FUI-A-05 | T-09-01 | typed deserialization | compile + suite | `cargo test --package shifty-dioxus` | ✅ (suite) | ⬜ pending |
| 9-02-01 | 02 | 2 | FUI-A-05 | T-09-04/05/06 | rollback all close paths | suite | `cargo test --package shifty-dioxus` | ✅ (suite) | ⬜ pending |
| 9-02-02 | 02 | 2 | FUI-A-05 (SC2/SC3) | — | N/A | source self-test grep | `cargo test --package shifty-dioxus no_copy_week_in_frontend_source` | ❌ W0 (creates) | ⬜ pending |
| 9-02-03 | 02 | 2 | FUI-A-05 | — | N/A | WASM + workspace gate | `cargo build --target wasm32-unknown-unknown` + `cargo check --workspace` | ✅ (gate) | ⬜ pending |
| 9-02-04 | 02 | 2 | FUI-A-05 (SC1) | T-09-04/05 | live rollback UX | MANUAL checkpoint | none (see Manual-Only) | n/a | ⬜ pending |

*No 3 consecutive tasks without an automated verify. Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] SSR-snapshot tests per warning variant (`BookingOnAbsenceDay`, `BookingOnUnavailableDay`, `PaidEmployeeLimitExceeded`) — created in `component/warning_list.rs` (Plan 01 Task 2)
- [ ] Empty-warnings ⇒ no dialog assertion (`warning_list_empty_renders_nothing`) — Plan 01 Task 2
- [ ] Rollback action-dispatch coverage (Abbrechen → DELETE) — exercised by Plan 02 Task 1 handler wiring; verified via suite + manual UAT (live coroutine dispatch is not SSR-testable, see Manual-Only)
- [ ] Per-Locale-Reference-Matcher tests for new i18n keys (Pitfall-2 guard) — `i18n_booking_warning_keys_match_german_reference` (Plan 01 Task 1)
- [ ] i18n parity test extension — `i18n_booking_warning_keys_present_in_all_locales` (Plan 01 Task 1)
- [ ] Copy-week reintroduction self-test — `no_copy_week_in_frontend_source` (Plan 02 Task 2)

*Existing test infrastructure (SSR via VirtualDom + dioxus_ssr, `pin_de_locale()` hook) covers the framework; only new test files/cases needed. Wave 0 stubs are created in Wave 1 (Plan 01) before consumers in Wave 2 (Plan 02).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live confirm-dialog UX (open/confirm/cancel against running backend) | FUI-A-05 (SC1) | Requires backend + WASM runtime + coroutine dispatch | Book onto an absence/unavailable day; verify Dioxus dialog appears; "Abbrechen"/ESC/backdrop roll back (DELETE), "Trotzdem buchen" keeps. Plan 02 Task 4 checkpoint. |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies (Task 9-02-04 is the documented manual-only exception)
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** planner-approved 2026-06-12
