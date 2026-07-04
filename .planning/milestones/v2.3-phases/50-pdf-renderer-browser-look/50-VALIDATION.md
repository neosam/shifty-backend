---
phase: 50
slug: pdf-renderer-browser-look
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-03
---

# Phase 50 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `50-RESEARCH.md` §"Validation Architecture" — do not duplicate; treat as canonical companion.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `#[tokio::test]` (via `service_impl` unit-tests, `#[cfg(test)] mod test` in `pdf_render.rs`) |
| **Config file** | Cargo standard (kein separater Config) |
| **Quick run command** | `cargo test -p service_impl pdf_render -- --nocapture` |
| **Full suite command** | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |
| **Estimated runtime** | Quick ~15s (nur `pdf_render`-Modul), Full ~90s (Workspace) |

---

## Sampling Rate

- **After every task commit:** `cargo test -p service_impl pdf_render && cargo clippy --workspace -- -D warnings`
- **After every plan wave:** `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite grün + UAT (D-50-17: manueller Klick auf Phase-49-Button, visuelle Kontrolle im Browser)
- **Max feedback latency:** ~15 Sekunden (Quick-Suite)

---

## Per-Task Verification Map

*Die konkrete Task-→-Requirement-Zuordnung entsteht im Planning-Schritt. Diese Tabelle wird beim Wave-0-Sign-Off befüllt, sobald `50-PLAN.md`-Dateien existieren.*

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 50-01-XX | 01 | 1 | PDF-01/02 | — | N/A (kein Threat-Model für pure-fn Rewrite) | unit | `cargo test -p service_impl pdf_render` | ⚠️ Wave 0 legt Tests an | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `service_impl/Cargo.toml`: `time`-Features um `"local-offset"` erweitern (Research §"Standard Stack" / Pitfall 2).
- [ ] Neue Test-Funktionen in `service_impl/src/pdf_render.rs` (`#[cfg(test)] mod test`) laut D-50-16:
  - `render_includes_timestamp_string`
  - `slot_boxes_sorted_by_start_time`
  - `names_within_slot_alphabetical`
  - `unpaid_marker_suffix`
  - `sunday_column_hidden_when_no_sunday_slots`
  - `sunday_column_shown_when_at_least_one_sunday_slot`
- [ ] Neuer Service-Level-Test in `service_impl/src/test/pdf_shiftplan.rs` (oder analoges Test-Modul):
  - `now_local_fallback_to_utc_on_indeterminate_offset`
- [ ] Fixture-Extension: `make_sales_person(...)` bekommt `is_paid: Option<bool>`-Parameter (für D-50-07-Suffix-Test).
- [ ] Fixed-Timestamp-Konstante `FIXED_RENDER_TIMESTAMP: OffsetDateTime` als Test-Konstante deklarieren (D-50-14).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visueller Look entspricht Browser-Wochenansicht | PDF-01 | Pixel-Perfect nicht automatisierbar (Overkill via headless Chrome für v2.3). | D-50-17: `dx serve` starten, Backend läuft, User navigiert zu einer Woche mit `Planned`/`Locked` Status, klickt PDF-Download-Button (Phase-49), öffnet PDF, prüft: (a) Slots als sichtbare Boxen in Tages-Spalten, (b) `08:00 - 12:00`-Zeit-Labels vorhanden, (c) Sales-Person-Namen in Boxen alphabetisch, (d) Kopfzeile `Schichtplan KW NN (JJJJ)` oben-links, (e) Timestamp `Erstellt am DD.MM.YYYY HH:MM Uhr` oben-rechts, (f) Sonntag-Spalte nur bei Sonntag-Slot. |

---

## Validation Sign-Off

- [ ] All tasks have `<acceptance_criteria>` mit Test-Command oder Wave-0-Dependency (per gsd-planner)
- [ ] Sampling continuity: pdf_render-Modul-Test läuft nach jedem Task-Commit
- [ ] Wave 0 covers all MISSING references (siehe „Wave 0 Requirements" oben)
- [ ] No watch-mode flags (Cargo-Tests laufen einmalig)
- [ ] Feedback latency < 20s pro Quick-Run
- [ ] `nyquist_compliant: true` set in frontmatter (nach Wave-0-Sign-Off)

**Approval:** pending
