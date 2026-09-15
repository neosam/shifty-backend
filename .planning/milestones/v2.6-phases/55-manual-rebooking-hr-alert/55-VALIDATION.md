---
phase: 55
slug: manual-rebooking-hr-alert
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-23
last_updated: 2026-07-23
plan_count: 0
---

# Phase 55 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Quelle: `55-RESEARCH.md` §Validation Architecture. Wird beim Planning-Wrap-Commit
> pro Task in PLAN.md konkretisiert (Task-IDs entstehen erst im Planner).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (`#[tokio::test]` async) + `mockall` (Unit) + in-memory SQLite (Integration) |
| **Config file** | keine separate — Cargo-nativ (`Cargo.toml` workspace root) |
| **Quick run command** | `cargo test -p service_impl rebooking` |
| **Full suite command** | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |
| **Estimated runtime** | ~30–60 s (quick, pfad-scoped) / ~180–240 s (full + Clippy) |

---

## Sampling Rate

- **After every task commit:** `cargo test -p service_impl rebooking` + `cargo clippy --workspace -- -D warnings`
- **After every plan wave:** Full suite (`cargo test --workspace && cargo clippy --workspace -- -D warnings`)
- **Before `/gsd-verify-work`:** Full suite green + Clippy green (`nix build`-Gate) + FE `cargo build --target wasm32-unknown-unknown` + FE-Clippy green
- **Max feedback latency:** 240 s

---

## Per-Task Verification Map

Requirement-scoped (Task-IDs werden vom Planner vergeben und beim Planning-Wrap-Commit
eingetragen). REQ-IDs aus ROADMAP.md §Phase 55.

| Requirement | Behavior | Test Type | Automated Command | File Exists | Status |
|-------------|----------|-----------|-------------------|-------------|--------|
| VOL-ACCT-03 | Roundtrip-Neutralität (balance/ist/soll/delta before==after) — **TDD ZUERST** | Property/Integration | `cargo test -p service_impl rebooking_neutrality` | ❌ W0 (`service_impl/src/test/rebooking_reconciliation.rs`) | ⬜ pending |
| REB-MANUAL-01 | Atomarer Doppel-Eintrag (2×ExtraHours + Batch + Entry) in EINER Tx | Integration | `cargo test -p service_impl rebooking_reconciliation_creates_pair` | ❌ W0 | ⬜ pending |
| REB-MANUAL-02 | Beide Richtungen (VolunteerWork↔ExtraWork) | Unit/Integration | `cargo test -p service_impl rebooking_direction` | ❌ W0 | ⬜ pending |
| REB-MANUAL-03 | Modal-Vorschau — Fat-Backend: `proposed_hours`-fn testbar | Unit + UAT | `cargo test -p service_impl proposed_hours` | ❌ W0 | ⬜ pending |
| HR-ALERT-01 | Alert-Predicate `balance<=-0.5 ∧ vol_ist>0 ∧ cap` (pure fn) | Unit | `cargo test -p service_impl alert_predicate` | ❌ W0 | ⬜ pending |
| HR-ALERT-02 | IST/DANN backend-computed (`RebookingSuggestionTO`, kein FE-Arithmetik) | Integration | `cargo test -p service_impl suggestion_dann_values` | ❌ W0 | ⬜ pending |
| HR-ALERT-03 | Approve/Reject state-conditional UPDATE (`WHERE state='pending'`, rows_affected==1, Race) | Integration | `cargo test -p service_impl approve_double_race_conflict` | ❌ W0 | ⬜ pending |
| HR-ALERT-04 | UNIQUE-Slot-Claim (409/Conflict bei Kollision) | Integration | `cargo test -p service_impl rebooking_slot_taken_conflict` | ❌ W0 | ⬜ pending |
| D-55-01 | Threshold-Grenzfälle -0.49 / -0.5 / -0.51 | Unit | `cargo test -p service_impl alert_predicate_boundary` | ❌ W0 | ⬜ pending |
| D-55-03 | `min(\|balance\|, voluntary_ist)`-Matrix inkl. vol_ist==0 → kein Alert | Unit | `cargo test -p service_impl proposed_hours` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `service_impl/src/test/rebooking_reconciliation.rs` — REB-MANUAL-*, HR-ALERT-03/04, VOL-ACCT-03
- [ ] Pure-fn-Tests für `alert_predicate` + `proposed_hours` (im `reporting.rs`-Test-Modul oder eigenem Modul)
- [ ] `service_impl/src/test/mod.rs` — neues Test-Modul registrieren
- [ ] Mocks: `MockRebookingBatchService`, `MockExtraHoursService` bereits via `automock` vorhanden — kein Setup-Bedarf
- [ ] **VOL-ACCT-03-Property-Test ZUERST** (tdd_mode) — nagelt die Neutralitäts-Invariante fest, BEVOR der `source`-Filter in `reporting.rs` entsteht (Research-Kritikbefund: Filter noch nicht aktiv)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Alert-Banner Render/Hide, Modal-Interaktion (IST/DANN, Approve/Reject-Buttons), F3-TopBar-Trigger | REB-MANUAL-03, HR-ALERT-01..04 | Dioxus/WASM-UI nicht zuverlässig pixel-/interaktions-automatisierbar (Pitfall 11, MEMORY reference_dioxus_browser_test_date_inputs) | Live-HR-Smoke: gedeckelte Person mit balance<0 ∧ vol_ist>0 → Banner sichtbar; Klick → Modal DANN-Werte; Approve → Banner weg + 2 ExtraHours + Batch; Reject → Banner weg, state=rejected sichtbar |

Struktur (Predicate, DANN-Werte, State-Machine) ist über Backend-Pure-fn- + Integration-Tests voll abgedeckt; nur die visuelle Dioxus-Interaktion bleibt manuell.

---

## Validation Sign-Off

- [ ] Alle Tasks haben `<automated>` verify oder Wave-0-Dependency
- [ ] Sampling-Kontinuität: keine 3 aufeinanderfolgenden Tasks ohne automated verify
- [ ] Wave 0 deckt alle MISSING-Referenzen
- [ ] Keine watch-mode-Flags
- [ ] Feedback-Latenz < 240 s
- [ ] `nyquist_compliant: true` gesetzt (nach Planner-Alignment)

**Approval:** pending
