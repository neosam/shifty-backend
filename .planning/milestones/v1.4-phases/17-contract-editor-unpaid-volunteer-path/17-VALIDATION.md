---
phase: 17
slug: contract-editor-unpaid-volunteer-path
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-24
---

# Phase 17 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (mockall + in-memory SQLite) backend; `cargo build --target wasm32-unknown-unknown` Frontend-Gate |
| **Config file** | none — Workspace-Cargo + `service_impl/src/test/` Module |
| **Quick run command** | `cargo test -p service_impl reporting` (oder gezieltes Modul) |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~60–120 seconds (Backend); WASM-Build ~zusätzlich |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p <crate> <module>` für die berührte Site
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** `cargo test --workspace` grün + `cargo build --target wasm32-unknown-unknown` grün (aus `shifty-dioxus/`)
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

> Wird vom Planner verfeinert; hier die validierungs-kritischen Invarianten aus dem RESEARCH § Validation Architecture.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 17-XX-XX | XX | 1 | CVC-10 | — | `get_week` liefert KEINE unbezahlte Person (`is_paid=false`) im `WorkingHoursPerSalesPerson`-Set — kein Personen-Set-Leak | integration | `cargo test -p service_impl get_week` | ❌ W0 | ⬜ pending |
| 17-XX-XX | XX | 1 | CVC-10 | — | Personen-Set-Konsistenz: year-summary / all-employees-report / Billing enthalten dieselbe paid-Menge | integration | `cargo test -p service_impl` | ❌ W0 | ⬜ pending |
| 17-XX-XX | XX | 2 | CVC-09 | — | Open→Save-unverändert-Round-Trip bewahrt `committed_voluntary` (beide TryFrom-Richtungen) | unit | `cargo test -p <frontend-state-crate> employee_work_details` | ❌ W0 | ⬜ pending |
| 17-XX-XX | XX | 2 | CVC-10 | — | D-05 Read-Gate (`cap \|\| expected_hours == 0`) zählt unbezahlte Freiwillige in `get_weekly_summary`-Kapazität | integration | `cargo test -p service_impl get_weekly_summary` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `service_impl/src/test/` — `get_week`-Seiteneffekt-Integrationstest (no-leak + Personen-Set-Konsistenz) als Stub für CVC-10
- [ ] Erweiterte/neue Fixtures: unbezahlte Person (`is_paid=false`, `expected_hours=0`, `committed_voluntary>0`) mit aktivem Vertrag in der Test-Woche
- [ ] `get_weekly_summary`-Fixture für D-05-Gate-Pfad (`expected_hours == 0`-Zweig)

*Framework vorhanden — keine Installation nötig.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Editor-Feld sichtbar/editierbar bei `cap == true` ODER `expected_hours == 0` | CVC-09 | Dioxus-UI-Sichtbarkeitslogik — programmatisches Setzen von Inputs triggert Signale unzuverlässig (numerisch hier weniger kritisch, aber UI-Render manuell prüfen) | Vertrags-Editor öffnen für gedeckelte + 0-Soll-Person; Feld sichtbar; bei ungedeckelter Soll>0-Person ausgeblendet |
| „alle"-Filter blendet unbezahlte Freiwillige ein | CVC-10 | Frontend-Filter + Backend-Merge-Pfad; visuelles Toggle-Verhalten | Mitarbeiteransicht: Default nur bezahlt; „alle"-Toggle → unbezahlte nicht-inaktive Freiwillige erscheinen |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
