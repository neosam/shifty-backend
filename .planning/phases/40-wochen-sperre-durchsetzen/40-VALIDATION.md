---
phase: 40
slug: wochen-sperre-durchsetzen
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-02
---

# Phase 40 — Validation Strategy

> Per-phase validation contract. Derived from 40-RESEARCH.md "## Validation Architecture".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in + `mockall` (backend); `cargo test` (FE) |
| **Config file** | none — workspace `Cargo.toml` |
| **Quick run command** | `cargo test -p service_impl shiftplan_edit_lock` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~60–120 seconds |

---

## Sampling Rate

- **After every task commit:** `cargo test -p service_impl shiftplan_edit_lock` + `cargo clippy --workspace -- -D warnings`
- **After every plan wave:** `cargo test --workspace` + `cargo build --target wasm32-unknown-unknown` (shifty-dioxus)
- **Before `/gsd-verify-work`:** Full suite green (sqlx prepare only if a new query appears — not expected)
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map — 6 write paths × {Locked, Open}

| Test-ID | Method | Week | Role | Expected |
|---------|--------|------|------|----------|
| T-40-01 | modify_slot | Locked | non-shiftplanner | Err(WeekLocked) — but note modify_slot needs shiftplan.edit → Forbidden first; gate relevance limited |
| T-40-02 | modify_slot | Locked | shiftplanner | Ok (bypass) |
| T-40-04 | modify_slot_single_week | Locked | shiftplanner | Ok (bypass) |
| T-40-05/06 | remove_slot | Locked/Open | shiftplanner | Ok (bypass) |
| T-40-07 | book_slot | Locked | non-shiftplanner | Err(WeekLocked { year, week }) |
| T-40-08 | book_slot | Locked | shiftplanner | Ok (bypass) |
| T-40-09 | book_slot | Open | non-shiftplanner | Ok |
| T-40-10 | copy_week | Locked target | shiftplanner | Ok (bypass) |
| T-40-11 | copy_week | Locked source, Open target | shiftplanner | Ok (target week is the writing week) |
| T-40-12 | delete_booking | Locked | non-shiftplanner | Err(WeekLocked) — closes the WST-04 bypass |
| T-40-13 | delete_booking | Locked | shiftplanner | Ok (bypass) |
| T-40-14 | delete_booking | Open | non-shiftplanner | Ok |
| T-40-15 | delete_booking | non-existent id | any | Err(EntityNotFound) before lock gate |
| T-40-16 | in-transaction (no TOCTOU) | Locked | non-shiftplanner | write mock NOT called (mockall expect count 0) — no write effect before the lock gate |
| T-40-17 | delete_booking order | Locked | non-shiftplanner | booking_service.delete NOT called; get→lock-check→(blocked) |

All backend tests: integration with in-memory SQLite + `mockall` for WeekStatusService/PermissionService, in the style of `service_impl/src/test/week_status.rs`.

**FE:** the +/- buttons are hidden for non-shiftplanner when `week_status == Locked` — pure predicate / SSR-style check where mountable (D-40-03). The lock state reads from the already-loaded Phase-39 status, not the 423.

---

## Wave 0 Requirements

- [ ] `service_impl/src/test/shiftplan_edit_lock.rs` — new test module for the 6-path matrix + TOCTOU + delete_booking-order tests
- [ ] `service_impl/src/test/mod.rs` — register `mod shiftplan_edit_lock;`

Existing `week_status` and `shiftplan_edit` tests remain unchanged.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live browser: +/- buttons vanish for non-shiftplanner in a Locked week; red "Gesperrt" badge visible | WST-03 (FE) | D-25-06 class — WASM signal interaction | Optional smoke; structural coverage (button_mode predicate) is the hard gate |

*All server-enforcement behaviors have automated verification (the 423 gate is fully unit/integration-tested).*

---

## Validation Sign-Off

- [ ] All 6 write paths covered by the lock matrix (5 existing + delete_booking)
- [ ] In-transaction / no-TOCTOU test present (T-40-16)
- [ ] delete_booking bypass-closure test present (T-40-12, T-40-17)
- [ ] 423 mapping asserted (WeekLocked → HTTP 423)
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
