---
phase: 23
slug: frontend-slot-paid-capacity-ui
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-26
---

# Phase 23 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust) + `dioxus-ssr 0.6` SSR snapshot rendering (dev-dependency, `shifty-dioxus/Cargo.toml`) |
| **Config file** | none — existing infrastructure (`shifty-dioxus/Cargo.toml`) |
| **Quick run command** | `cd shifty-dioxus && cargo test slot_edit week_view` |
| **Full suite command** | `cd shifty-dioxus && cargo test` |
| **Estimated runtime** | ~30–60 seconds |

**Build gate (mandatory before sign-off):** `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` (WASM-Build-Gate) + `cargo clippy --workspace -- -D warnings`.

---

## Sampling Rate

- **After every task commit:** Run `cd shifty-dioxus && cargo test slot_edit week_view`
- **After every plan wave:** Run `cd shifty-dioxus && cargo test`
- **Before `/gsd-verify-work`:** Full suite green + WASM build green + clippy clean
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 23-01-xx | 01 | 1 | D-23-03/04 (`bad`-Färbung + Vorrang) | — | N/A (read-only display) | unit | `cd shifty-dioxus && cargo test cell_background_class` | ✅ existing | ⬜ pending |
| 23-01-xx | 01 | 1 | D-23-03 (SSR-Zelle `bg-bad-soft`) | — | N/A | unit (SSR) | `cd shifty-dioxus && cargo test week_view` | ✅ existing | ⬜ pending |
| 23-02-xx | 02 | 1 | D-23-01 (Editor-Feld + NULL) | — | N/A | unit (SSR) | `cd shifty-dioxus && cargo test slot_edit` | ✅ existing | ⬜ pending |
| 23-02-xx | 02 | 1 | D-23-02 (Inline-Hinweis Limit<count) | — | N/A | unit (SSR) | `cd shifty-dioxus && cargo test slot_edit` | ✅ existing | ⬜ pending |
| 23-02-xx | 02 | 1 | D-23-06 (i18n De/En/Cs Keys) | — | N/A | unit | `cd shifty-dioxus && cargo test i18n` / build | ✅ existing | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.* `dioxus-ssr 0.6` is already a dev-dependency; SSR snapshot pattern (`VirtualDom::new` → `rebuild_in_place` → `dioxus_ssr::render`) is canonically used in `warning_list.rs` tests. No framework install or new fixtures required.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Tatsächliche Farbwirkung im Browser (rot vs. orange, light+dark) | D-23-03 | SSR-Test prüft Klassen-Namen, nicht die gerenderte Farbe/Kontrast | Backend+Frontend starten, Slot mit Limit < current_paid_count im Week-View ansehen (light & dark mode) |
| Inline-Banner-Platzierung & Lesbarkeit im Dialog | D-23-02 | Visuelle Positionierung ist subjektiv | Slot-Editor öffnen, Limit unter current setzen, Banner-Position prüfen |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (none required)
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
