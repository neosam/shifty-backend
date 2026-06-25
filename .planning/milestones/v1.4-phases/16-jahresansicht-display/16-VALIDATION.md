---
phase: 16
slug: jahresansicht-display
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-24
---

# Phase 16 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Quelle: `16-RESEARCH.md` § Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in; Backend-Workspace + Frontend `shifty-dioxus/` getrennt) |
| **Config file** | none — bestehende Test-Infrastruktur (Backend `service_impl`, Frontend `dioxus-ssr` SSR-Render-Tests) |
| **Quick run command** | `cargo test -p shifty-dioxus` (Frontend) / `cargo test -p service_impl booking_information` (Backend) |
| **Full suite command** | `cargo test --workspace` (Backend) + `cargo test` in `shifty-dioxus/` + WASM-Gate |
| **WASM-Build-Gate** | `cargo build --target wasm32-unknown-unknown` (in `shifty-dioxus/`, unter `nix develop`) |
| **Estimated runtime** | ~60–120 Sekunden (Workspace) |

---

## Sampling Rate

- **After every task commit:** `cargo test -p shifty-dioxus` (Frontend-Tasks) bzw. `cargo test -p service_impl booking_information` (Backend-Tasks)
- **After every plan wave:** `cargo test --workspace` + `cargo test` in `shifty-dioxus/` + WASM-Build-Gate
- **Before `/gsd-verify-work`:** Volle Suite grün + WASM-Gate exit 0
- **Max feedback latency:** ~120 Sekunden

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 16-01-* | 01 | 1 | CVC-07 (a) | — | N/A | unit | `cargo test -p service_impl booking_information` | ✅ (bestehend, neue Tests) | ⬜ pending |
| 16-01-* | 01 | 1 | CVC-07 (b) | — | N/A | unit (From-Roundtrip) | `cargo test -p rest-types` / `-p service_impl` | ❌ W0 | ⬜ pending |
| 16-02-* | 02 | 2 | CVC-07 (c) | — | N/A | unit | `cargo test -p shifty-dioxus` | ❌ W0 | ⬜ pending |
| 16-03-* | 03 | 3 | CVC-07 (d) | — | N/A | SSR-Render | `cargo test -p shifty-dioxus` | ❌ W0 | ⬜ pending |
| 16-03-* | 03 | 3 | CVC-07 (e) | — | N/A | SSR-Render | `cargo test -p shifty-dioxus` | ❌ W0 | ⬜ pending |
| 16-03-* | 03 | 3 | CVC-07 (f) | — | N/A | SSR-Render | `cargo test -p shifty-dioxus` | ❌ W0 | ⬜ pending |
| 16-03-* | 03 | 3 | CVC-07 (g) | — | N/A | SSR + Source-Audit | `cargo test -p shifty-dioxus` | ✅ (bestehend erweitert; Volunteer-Guard verengt) | ⬜ pending |
| 16-03-* | 03 | 3 | CVC-07 (h) | — | N/A | unit | `cargo test -p shifty-dioxus` | ❌ W0 | ⬜ pending |
| 16-03-* | 03 | 3 | CVC-08 (a–d) | — | N/A | unit (Per-Locale-Matcher) | `cargo test -p shifty-dioxus` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*
*Hinweis: Konkrete Task-IDs werden von den PLAN.md-Dateien gesetzt; diese Map bildet die Requirement→Test-Zuordnung ab.*

---

## Wave 0 Requirements

- [x] `rest-types/src/lib.rs` — From-Mapping-Test: `WeeklySummaryTO::from(&ws).committed_voluntary_hours == ws.committed_voluntary_hours` (Plan 16-01, Task TDD)
- [x] `shifty-dioxus/src/state/weekly_overview.rs` — From-Mapping-Roundtrip-Test (`committed_voluntary_hours` korrekt gemappt) (Plan 16-02, Task TDD)
- [x] `shifty-dioxus/src/page/weekly_overview.rs` — SSR-Test: dritter Token sichtbar; committed=5,actual=7 → `🎯5.00 | 🤝2.00`; committed=0 → `🎯0.00` (D-03) (Plan 16-03, Task 2 — behavior CVC-07d/e/f)
- [x] `shifty-dioxus/src/component/weekly_overview_chart.rs` — Test: `bar_total`/`compute_max_hours` nutzt `paid + committed + surplus`; drittes Segment kein Roh-Hex; bestehender `chart_volunteer_uses_ink_muted_not_good`-Guard auf segment-spezifisch verengt (D-04) (Plan 16-03, Task 3 — behavior CVC-07g/h + Schritt 8a)
- [x] `shifty-dioxus/src/i18n/{de,en,cs}.rs` — Per-Locale-Reference-Matcher-Tests für neue Keys; `Key::Volunteer` + `Key::PaidVolunteer`-Äquivalent in `cs.rs` (bestehende Lücken schließen) (Plan 16-03, Task 1 — behavior CVC-08a–d)

*Bestehende Test-Harnesse (`sample_week`-Helper, SSR-Suite, Hardcoded-Hex-Audit) werden erweitert, nicht neu gebaut. Alle Wave-0-Referenzen sind durch TDD-Tasks in den Plänen abgedeckt.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visuelle Drei-Farben-Stapelung des Charts (Farb-Differenzierung) | CVC-07 (g) | SSR rendert Klassen, nicht gerenderte Pixel; Farb-Lesbarkeit ist visuell | Jahresansicht im Browser öffnen, Balken auf drei unterscheidbare Segmente (paid/committed/surplus) prüfen, Tooltip nennt alle drei Werte |
| Czech-Übersetzungen sprachlich korrekt (`Přislíbeno`, `Dobrovolné`) | CVC-08 | Übersetzungsqualität nicht test-automatisierbar (A3 im Research) | User-Review der `cs.rs`-Strings |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved
