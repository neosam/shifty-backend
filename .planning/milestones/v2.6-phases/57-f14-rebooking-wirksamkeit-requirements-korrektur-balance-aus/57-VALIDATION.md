---
phase: 57
slug: f14-rebooking-wirksamkeit-requirements-korrektur-balance-aus
status: draft
nyquist_compliant: false
wave_0_complete: true
created: 2026-07-31
---

# Phase 57 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Abgeleitet aus `57-RESEARCH.md § Validation Architecture`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in Test-Harness + `tokio::test` (async) + `mockall` (Service-Mocks) |
| **Config file** | keins — Cargo-Workspace-Standard |
| **Quick run command** | `cargo test -p service_impl rebooking` |
| **Full suite command** | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |
| **Estimated runtime** | quick ~20–40 s · full ~3–6 min |

**Clippy ist hartes Gate** (CLAUDE.md): `cargo test` deckt Clippy NICHT ab.
`cargo clippy --workspace -- -D warnings` muss vor jedem Commit laufen.

---

## Sampling Rate

- **After every task commit:** `cargo test -p service_impl rebooking` + `cargo clippy --workspace -- -D warnings`
- **After every plan wave:** `cargo test --workspace`
- **Before `/gsd-verify-work`:** `cargo test --workspace && cargo clippy --workspace -- -D warnings` grün
- **Max feedback latency:** ~40 s (quick) / ~6 min (full)

---

## Kern-Invarianten (was diese Phase beweisen muss)

| # | Invariante | Warum sie zählt |
|---|-----------|-----------------|
| I-1 | `balance_after == balance_before + h` nach Umbuchung von `h` Stunden | Milestone-Goal v2.6; heute No-Op (−53 → −53 gemessen) |
| I-2 | `voluntary_ist_after == voluntary_ist_before − h` | Gegenbuchung; heute No-Op (3 → 3 gemessen) |
| I-3 | Auto-Cron: Balance steigt **nie** über 0 (`min(max(ist−committed,0), max(−balance,0))`) | REB-AUTO-06; heute überträgt die Formel bei 23 h/20 h fünf statt zwei Stunden |
| I-4 | Balance bereits ≥ 0 → **kein** Übertrag trotz Excess | Randfall des Deckels |
| I-5 | HR-Formel `min(\|balance\|, ist)` bleibt unverändert — darf die Zusage unterschreiten | Gewollt (User-bestätigt); kein Deckel auf dem HR-Pfad |
| I-6 | VOL-ACCT-01-Soll bleibt rebooking-blind | VOL-ACCT-03 (Neufassung) schützt die Freiwilligen-*Statistik*, nicht die Balance |
| I-7 | `backfill_range`-Preview zeigt denselben gedeckelten Wert wie `run_for_week` schreibt | Kritischer Research-Fund: `week_excess` hat **zwei** Call-Sites |
| I-8 | Perioden ohne Rebooking-Woche bleiben byte-identisch; Toggle-Unabhängigkeit pre/post/straddling erhalten | D-57-11, Snapshot-Bump 12→13 |

---

## Per-Task Verification Map

*Task-IDs werden beim Planen vergeben; die Zeilen unten binden Requirement → Test → Kommando.
Der Executor trägt Task-IDs und Status nach.*

| Task ID | Plan | Wave | Requirement | Invariante | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------|-------------------|-------------|--------|
| TBD | 01 | 1 | VOL-ACCT-03 (Neufassung) | I-6 | doc/spec | `grep -q` auf `.planning/REQUIREMENTS.md` | ✅ | ⬜ pending |
| TBD | 01 | 1 | VOL-ACCT-04 (neu) | I-1, I-2 | doc/spec | `grep -q` auf `.planning/REQUIREMENTS.md` | ✅ | ⬜ pending |
| TBD | 01 | 1 | REB-AUTO-06 (neu) | I-3 | doc/spec | `grep -q` auf `.planning/REQUIREMENTS.md` | ✅ | ⬜ pending |
| TBD | 02 | 2 | VOL-ACCT-04 | I-1 | unit | `cargo test -p service_impl test_neutrality_balance` | ✅ (invertieren) | ⬜ pending |
| TBD | 02 | 2 | VOL-ACCT-04 | I-2 | unit | `cargo test -p service_impl test_neutrality_f1_ist` | ✅ (invertieren) | ⬜ pending |
| TBD | 02 | 2 | VOL-ACCT-03 | I-6 | unit | `cargo test -p service_impl test_neutrality_f2_soll` | ✅ (unverändert) | ⬜ pending |
| TBD | 02 | 2 | VOL-ACCT-04 | I-1, I-2 | unit | `cargo test -p service_impl rebooking_neutrality_single_manual` | ✅ (invertieren) | ⬜ pending |
| TBD | 02 | 2 | VOL-ACCT-04 | I-1, I-2 | unit | `cargo test -p service_impl rebooking_neutrality_roundtrip` | ✅ (unverändert) | ⬜ pending |
| TBD | 02 | 2 | VOL-ACCT-04 | I-1 | unit | Weekly-Summary-Test auf `assemble_weeks` (Site 1) | ❌ neu | ⬜ pending |
| TBD | 03 | 2 | REB-AUTO-06 | I-3 | unit | `cargo test -p service_impl run_for_week` (neuer Case) | ❌ neu | ⬜ pending |
| TBD | 03 | 2 | REB-AUTO-06 | I-4 | unit | `week_excess_matrix`-Erweiterung in `test/voluntary_rebooking.rs` | ✅ (erweitern) | ⬜ pending |
| TBD | 03 | 2 | REB-AUTO-06 | I-7 | unit | `cargo test -p service_impl -- rebooking_backfill` | ✅ (erweitern) | ⬜ pending |
| TBD | 03 | 2 | REB-AUTO-06 | I-5 | unit | HR-Pfad `suggest()` — bestehende Tests müssen grün bleiben | ✅ | ⬜ pending |
| TBD | 04 | 3 | VOL-ACCT-04 | I-1, I-2 | integration | Referenz-Szenario D-57-02 (5+5 h Vertrag, 3 h / 17 h) | ❌ neu | ⬜ pending |
| TBD | 04 | 3 | Snapshot 12→13 | I-8 | unit | `cargo test -p service_impl billing_period_straddling` | ✅ (Fixture erweitern) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

**Keine.** Alle benötigten Test-Dateien existieren bereits:

- `service_impl/src/test/rebooking_neutrality.rs`
- `service_impl/src/test/rebooking_reconciliation.rs`
- `service_impl/src/test/voluntary_rebooking.rs`
- `service_impl/src/test/rebooking_backfill.rs`
- `service_impl/src/test/billing_period_straddling.rs`

Alle benötigten Mock-Harnesses sind im Workspace verfügbar
(`MockReportingService` — inkl. des heute ungenutzten `reporting_service(balance: f32)`,
`MockExtraHoursService`, `MockShiftplanReportService`).
Kein neues Test-Framework, keine neue Fixture-Infrastruktur nötig.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| HR-Alert-Banner erlischt bei Approve UND bei Reject (D-57-04 — bleibt bewusst so) | — | Frontend-Verhalten, Dioxus-UI ohne E2E-Harness | Umbuchung vorschlagen → Banner sichtbar → Reject → Banner weg; Gegenprobe Approve |
| `suggest()`-Vorschau (`balance_after`, `voluntary_ist_after`) stimmt nach dem Fix mit dem tatsächlichen Ergebnis überein | VOL-ACCT-04 | Verspricht heute Werte, die nie eintreten; Ende-zu-Ende über REST + UI | HR-Modal öffnen, `balance_after` notieren, Approve, Employee-Report neu laden, Balance vergleichen |

*Kein Frontend-Code-Diff in dieser Phase (RESEARCH.md: alle betroffenen DTO-Felder
werden bereits geliefert und werden nach dem Fix nur numerisch korrekt).*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references *(n/a — keine Wave-0-Lücken)*
- [ ] No watch-mode flags
- [ ] Feedback latency < 40s (quick) / < 6min (full)
- [ ] `cargo clippy --workspace -- -D warnings` in jedem Task-Gate enthalten
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
