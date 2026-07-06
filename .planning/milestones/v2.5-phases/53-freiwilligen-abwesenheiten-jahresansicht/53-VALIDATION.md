---
phase: 53
slug: freiwilligen-abwesenheiten-jahresansicht
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-06
---

# Phase 53 — Validation Strategy

> Per-phase validation contract for feedback sampling während execution.
> Abgeleitet aus 53-RESEARCH.md `## Validation Architecture`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`#[test]`) + tokio (`#[tokio::test]`); Mock-Doubles im `service_impl/src/test/`-Baum |
| **Config file** | keine — Cargo.toml Workspace-Konfig |
| **Quick run command** | `cargo test -p service_impl booking_information_vaa -- --nocapture` |
| **Full suite command** | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |
| **Estimated runtime** | ~35–50 s (Backend full) + WASM-Gate ~30 s (FE) |

---

## Sampling Rate

- **After every task commit:** `cargo test -p service_impl booking_information` (~10 s)
- **After every plan wave:** `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite grün + FE-WASM-Gate `cargo build --target wasm32-unknown-unknown` in `shifty-dioxus/`
- **Max feedback latency:** < 60 s per task-commit

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 53-01-01 | 01 | 1 | VAA-01 (DTO-Schicht) | — | N/A (reine DTO-Erweiterung) | unit (compile) | `cargo build -p rest-types` | ❌ W0 | ⬜ pending |
| 53-01-02 | 01 | 1 | VAA-01 (Service-Struct) | — | N/A | unit (compile) | `cargo build -p service` | ❌ W0 | ⬜ pending |
| 53-01-03 | 01 | 1 | VAA-01 (From-Impl) | — | N/A | unit (compile) | `cargo build -p rest-types` | ❌ W0 | ⬜ pending |
| 53-02-01 | 02 | 2 | VAA-01+VAA-02 (Fill-Site 1) | — | Endpoint erfordert SHIFTPLANNER (bestehend) | unit | `cargo test -p service_impl booking_information_vaa` | ❌ W0 | ⬜ pending |
| 53-02-02 | 02 | 2 | VAA-01+VAA-02 (Fill-Site 2 — Single-Week) | — | Endpoint erfordert SHIFTPLANNER (bestehend) | unit | `cargo test -p service_impl booking_information_vaa` | ❌ W0 | ⬜ pending |
| 53-02-03 | 02 | 2 | VAA-03 #1 | — | N/A | unit | `cargo test -p service_impl vaa03_volunteer_with_period` | ❌ W0 | ⬜ pending |
| 53-02-04 | 02 | 2 | VAA-03 #2 | — | N/A | unit | `cargo test -p service_impl vaa03_volunteer_without_period_absent` | ❌ W0 | ⬜ pending |
| 53-02-05 | 02 | 2 | VAA-03 #3 (Regression-Lock) | — | N/A | unit | `cargo test -p service_impl vaa03_paid_employee_unchanged` | ❌ W0 | ⬜ pending |
| 53-03-01 | 03 | 3 | VAA-04 (FE-Union-Merge) | — | N/A | unit (FE) | `cargo test -p shifty-dioxus state::weekly_overview` | ❌ W0 | ⬜ pending |
| 53-03-02 | 03 | 3 | VAA-04 (Rendering-Lock) | — | N/A | static grep | `grep -n '"{name}: {hours} h"' shifty-dioxus/src/page/weekly_overview.rs` | ✅ | ⬜ pending |
| 53-03-03 | 03 | 3 | VAA-04 (WASM-Build) | — | N/A | build | `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*W0 = Testdatei muss in Wave 0 (siehe unten) angelegt werden, bevor der zugehörige Task laufen kann.*

---

## Wave 0 Requirements

- [ ] `service_impl/src/test/booking_information_vaa.rs` — neue Test-Datei mit VAA-01/02/03-Assertions (`vaa03_volunteer_with_period_appears_with_correct_hours`, `vaa03_volunteer_without_period_absent_not_in_list`, `vaa03_paid_employee_unchanged_regression_lock`)
- [ ] `service_impl/src/test/mod.rs` — `pub mod booking_information_vaa;` eintragen
- [ ] `shifty-dioxus/src/state/weekly_overview.rs` — Union-Merge-Testmodul erweitern (Freiwilliger + Bezahlter in sort-order Assertion)

*Existing infrastructure covers:* `#[tokio::test]`, `mockall`, `NoneTypeExt` (aus `service_impl/src/test/`), Booking-Information-Test-Muster (`booking_information_vfa.rs` als Template).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Browser-Rendering: Freiwilliger + Bezahlter tauchen zusammen sortiert in der Wochen-Zeile auf (`/weekly_overview/`) | VAA-04 | Dioxus WASM Signal-Änderungen sind headless-schwer zu triggern (Memory: `reference_dioxus_browser_test_date_inputs`); Sichtprüfung ist Sekunden-schnell und deckt die visuelle Konsistenz mit ab | 1. Backend + FE starten (`start genossi`). 2. Testdaten: Freiwilliger mit Vacation-Period + Bezahlter mit `absence_hours`. 3. `/weekly_overview/` öffnen, Ziel-Woche prüfen: beide Namen erscheinen in einer alphabetisch sortierten Liste, Format `"{name}: {hours} h"`. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (`booking_information_vaa.rs`, `mod.rs`-Registrierung)
- [ ] No watch-mode flags
- [ ] Feedback latency < 60 s
- [ ] `nyquist_compliant: true` set in frontmatter (nach Sign-Off durch Planner + Executor)

**Approval:** pending
