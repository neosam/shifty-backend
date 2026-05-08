---
phase: 8
slug: absence-crud-page-foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-08
---

# Phase 8 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust workspace) + dioxus-ssr Snapshot (Frontend Component-Tests) |
| **Config file** | `Cargo.toml` (Backend Workspace), `shifty-dioxus/Cargo.toml` (Frontend) |
| **Quick run command** | `cargo test -p service_impl absence` (Backend Service-Layer) / `cargo test -p shifty-dioxus` (Frontend) |
| **Full suite command** | `cargo test --workspace` (Backend) + `cargo build --target wasm32-unknown-unknown` (Frontend WASM-Gate) |
| **Estimated runtime** | ~120 Sek (Backend full suite); ~60 Sek (WASM build); ~30 Sek (Frontend tests) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p service_impl <area>` für Backend-Tasks; `cargo build --target wasm32-unknown-unknown` für Frontend-Tasks
- **After every plan wave:** Run `cargo test --workspace` (Backend) + `cargo build --target wasm32-unknown-unknown` im `shifty-dioxus/` (Frontend)
- **Before `/gsd-verify-work`:** Full suite (`cargo test --workspace` + WASM build) muss green sein; UAT-Smoke (HR + Employee Login je einmal Anlage + Edit + Delete + Resturlaub) muss durchgelaufen sein
- **Max feedback latency:** 120 Sekunden (Backend full suite)

---

## Per-Task Verification Map

> Wird während `/gsd-plan-phase` und `/gsd-execute-phase` befüllt — pro PLAN.md `<task>` ein Eintrag mit Test-Command.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 8-01-XX | 01 | 1 | FUI-A-* / Backend | T-8-XX | hr ∨ self Permission, 422 Self-Overlap, 409 Version-Konflikt | unit | `cargo test -p service_impl vacation_balance` | ❌ W0 | ⬜ pending |
| 8-02-XX | 02 | 2 | FUI-A-01..04 | T-8-XX | Auth-Context-basierte Sicht | integration | `cargo build --target wasm32-unknown-unknown` | ✅ | ⬜ pending |
| 8-03-01 | 03 | 3 | (Plan-Backend-Surface-Lock) | T-8-SURFACE-DRIFT | OpenAPI surface gepinnt: Pfade + Schema-Namen + VacationBalance-Tag (version-agnostic) | integration | `cargo test -p rest --test openapi_surface` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Detaillierte Map wird in PLAN.md `<automated>` Blöcken angereichert; Plan-Phase befüllt Task-IDs analog `8-{plan_num}-{task_num}` Schema.*

---

## Wave 0 Requirements

- [ ] `service_impl/src/test/vacation_balance.rs` — Test-Stubs für VacationBalanceService (Read-Aggregate aus AbsencePeriods + WorkingHours)
- [ ] `rest/src/vacation_balance.rs` (oder Erweiterung in `absence.rs`) — Integration-Tests für REST-Layer (hr ∨ self, 200/403, OpenAPI-Schema-Match)
- [ ] `shifty-dioxus/src/page/absences.rs` — dioxus-ssr Snapshot-Test-Stub (Pattern: `dialog.rs:461`)
- [x] OpenAPI Surface-Assertion-Test (`rest/tests/openapi_surface.rs`) — version-agnostic Pfad-/Schema-Pinning (Plan 08-03 Option-B-Pivot, ersetzt das in commit `fdb70b5` entfernte Insta-Snapshot-Pattern; siehe 08-03-SUMMARY.md)

*Existing infrastructure: `service_impl/src/test/absence.rs` (1052 LOC) ist 1:1-Vorlage; `mockall`-Mocks + in-memory SQLite + `NoneTypeExt` etabliert.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| HR + Employee Login UAT-Smoke (Anlage + Edit + Delete + Resturlaubs-Anzeige) | FUI-A-01..04 + Goal-Success-Criterion 5 | Browser-Interaktion mit zwei Auth-Rollen | (1) Backend starten (`cargo run`); (2) Frontend starten (`dx serve --hot-reload` im `shifty-dioxus/`); (3) HR-User einloggen, neue Abwesenheit anlegen für anderen Mitarbeiter, editieren, löschen, Resturlaubs-Kachel prüfen; (4) Employee-User einloggen, eigene Abwesenheit anlegen, eigenen Resturlaub prüfen, fremde Liste darf nicht sichtbar sein. |
| AbsencePeriodCreateResultTO.warnings[] Rendering bei Booking-Konflikt | FUI-A-04 | Erfordert seeded Konflikt-Daten | Vacation-Range über existierende Booking legen, POST/PUT triggern, prüfen dass `WarningList` im Modal vor Close angezeigt wird, dann Acknowledge. |
| 409 Version-Konflikt-Banner | D-08 | Erfordert konkurrente Edit-Session | Zwei Tabs öffnen, in beiden gleicher Eintrag laden, in Tab 1 speichern, in Tab 2 speichern → Banner muss erscheinen mit "Erneut laden?"-Button. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (vacation_balance test file, REST integration test)
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter (after Plan-Phase + Wave 0 implementation)

**Approval:** pending
