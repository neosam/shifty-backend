---
phase: 08-absence-crud-page-foundation
plan: 06
status: partial
title: E2E-UAT-Smoke + Goal-Verification + Regression-Gates
date: 2026-05-08
---

## Outcome

**Task 1 (automated regression gates): COMPLETE.** Backend Full-Suite, WASM-Build, Frontend-Tests alle grün auf Stand `ddf60fd8` (= bb155f0c + Plan 08-09 + Dioxus.toml-Proxy-Fix).

**Tasks 2 + 3 (HR + Employee UAT-Smoke): DEFERRED.** Blockiert auf Migration der Legacy-`extra_hours` per Cutover. Cutover ist im int-Lauf am Drift-Gate gehängt — die Auto-Heuristik (Plan 08-09) deckt nicht alle realen Buchungs-Patterns ab. Strategische Entscheidung: Die manuelle Drift-Resolution wird über eine **Cutover-Migration-UI in Phase 9** gelöst, statt die Auto-Heuristik weiter zu erweitern.

UAT-Schritte sind als **HUMAN-UAT.md** (`08-HUMAN-UAT.md`, status `partial`) persistiert und tauchen in `/gsd-progress` und `/gsd-audit-uat` als Outstanding-Items auf, bis Phase 9 die Migration ermöglicht und der UAT auf int durchgezogen wird.

## Verification Gates (Task 1)

| Gate | Command | Result |
|------|---------|--------|
| Backend cargo check | `nix develop -c cargo check --workspace` | ✓ |
| Backend cargo test | `nix develop -c cargo test --workspace` | ✓ (488+ Tests, 0 failed) |
| Frontend cargo test (in shifty-dioxus/) | `cargo test` | ✓ (509+ tests, 0 failed) |
| Frontend WASM-Build (in shifty-dioxus/) | `cargo build --target wasm32-unknown-unknown` | ✓ |

## Goal-Success-Criteria-Mapping

| SC | Source | Status |
|----|--------|--------|
| SC-1 | FUI-A-01 — Route via Menü, HR-Privileg schaltet Filter | ⏸ deferred (UAT-Schritte 2+5 in HUMAN-UAT.md) |
| SC-2 | FUI-A-02+03 — CRUD AbsencePeriodTO + 422 Self-Overlap inline | ⏸ deferred (UAT-Schritte 8-11+19-20) |
| SC-3 | FUI-A-04 — Forward-Warnings als nicht-blockierende Liste | ⏸ deferred (UAT-Schritt 11) |
| SC-4 | D-03/D-04 — Backend-Resturlaubs-Endpoint + FE-Konsumption | ⏸ deferred (UAT-Schritt 4 HR + Employee) |
| SC-5 | WASM grün, Backend Full-Suite grün, UAT erfolgreich | ◑ partial — automated gates ✓, UAT deferred |

## Why Deferred (not Failed)

Phase 8 ist **funktional fertig**. Code, Tests, Build-Gates alle grün. Der int-UAT-Block ist eine **organisch gewachsene Folge-Anforderung**: Der Cutover-Algorithmus (v1.0 Phase 4) deckt nicht alle realen Buchungs-Konventionen ab, und die manuelle Drift-Resolution braucht eine UI. Diese Lücke gehört zu **Phase 9 — Cutover-Migration-UI**, nicht zu Phase 8.

Plan 08-09 (Wochenpauschalen-Heuristik, lokal getested) und Plan 08-08 (inline Drift-Report im REST-Response) wurden während dieser Phase eingeführt, um die Cutover-UX zu verbessern. Verbleibende Edge-Cases (Multi-Vertrags-Lebenszyklen, Teil-Wochen-Pauschalen, Feiertag-Inkonsistenz zwischen Pre-Check und Gate-Phase) werden in Phase 9 entweder durch Code-Fix oder durch UI-vermittelte manuelle Resolution geschlossen.

## Outstanding Items (Phase 9)

- HR-UAT-Smoke (20 Schritte) — siehe `08-HUMAN-UAT.md`
- Employee-UAT-Smoke (15 Schritte) — siehe `08-HUMAN-UAT.md`
- Forbidden-Test (Defense-in-Depth, T-8-AUTH-01 + T-8-IDOR-01) — UAT-Schritt 12 Employee
