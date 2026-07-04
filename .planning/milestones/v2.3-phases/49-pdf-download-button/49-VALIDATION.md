---
phase: 49
slug: pdf-download-button
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-03
---

# Phase 49 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `49-RESEARCH.md` §Validation Architecture (Zeile 702ff.).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Backend workspace + separater `shifty-dioxus` Workspace) |
| **Config file** | none — Cargo defaults; Nix-Build fährt zusätzlich `cargo clippy --workspace -- -D warnings` |
| **Quick run command** | `cargo test -p service_impl pdf_shiftplan -- --nocapture` |
| **Full suite command** | Backend: `cargo test --workspace && cargo clippy --workspace -- -D warnings`. Frontend: `cd shifty-dioxus && cargo test && cargo build --target wasm32-unknown-unknown` |
| **Estimated runtime** | Quick ~15s, Full ~90s (Backend) + ~60s (Frontend inkl. WASM-Build-Gate) |

---

## Sampling Rate

- **After every task commit:** `cargo test -p service_impl pdf_shiftplan` (+ `cargo test -p rest` falls Handler dorthin wandert)
- **After every plan wave:** Full backend suite + WASM-Build-Gate im Frontend
- **Before `/gsd-verify-work`:** Full suite grün + Browser-UAT-Klick auf den Button
- **Max feedback latency:** ~30s (Quick), ~150s (Full inkl. Frontend + WASM)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 49-01-01 | 01 | 1 | PDF-03 | — | REST `GET /shiftplan/{id}/{y}/{w}/pdf` liefert 200 + `application/pdf` + Content-Disposition-Header `attachment; filename="schichtplan-{yyyy}-KW{ww}.pdf"` | integration | `cargo test -p service_impl pdf_shiftplan::rest_returns_200_with_headers` | ❌ Wave 0 | ⬜ pending |
| 49-01-02 | 01 | 1 | PDF-03 | — | `PdfShiftplanService::render_week_pdf` happy-path liefert Bytes (Mock View + SalesPerson + WeekStatus=Planned) | unit | `cargo test -p service_impl pdf_shiftplan::happy_path_returns_bytes` | ❌ Wave 0 | ⬜ pending |
| 49-01-03 | 01 | 1 | PDF-03 | — | `PdfShiftplanService` filtert `deleted.is_some()` SalesPersons raus (Aufruf-Assertion via `MockPdfRender::expect_call`) | unit | `cargo test -p service_impl pdf_shiftplan::filters_deleted_sales_persons` | ❌ Wave 0 | ⬜ pending |
| 49-01-04 | 01 | 1 | PDF-03 | V5 | Filename-Format `schichtplan-{yyyy}-KW{ww:02}.pdf` im Content-Disposition | integration | `cargo test -p service_impl pdf_shiftplan::content_disposition_filename_format` | ❌ Wave 0 | ⬜ pending |
| 49-01-05 | 01 | 1 | PDF-04 | V5 | Backend: WeekStatus=Unset → 409 | integration | `cargo test -p service_impl pdf_shiftplan::week_status_unset_returns_409` | ❌ Wave 0 | ⬜ pending |
| 49-01-06 | 01 | 1 | PDF-04 | V5 | Backend: WeekStatus=InPlanning → 409 | integration | `cargo test -p service_impl pdf_shiftplan::week_status_in_planning_returns_409` | ❌ Wave 0 | ⬜ pending |
| 49-01-07 | 01 | 1 | PDF-04 | V5 | Backend: WeekStatus=Planned → 200 | integration | `cargo test -p service_impl pdf_shiftplan::week_status_planned_returns_200` | ❌ Wave 0 | ⬜ pending |
| 49-01-08 | 01 | 1 | PDF-04 | V5 | Backend: WeekStatus=Locked → 200 | integration | `cargo test -p service_impl pdf_shiftplan::week_status_locked_returns_200` | ❌ Wave 0 | ⬜ pending |
| 49-01-09 | 01 | 1 | PDF-04 | V5 | Service-internal Gate feuert bei Status-Race (Defense-in-Depth) | unit | `cargo test -p service_impl pdf_shiftplan::service_defense_in_depth_rejects` | ❌ Wave 0 | ⬜ pending |
| 49-01-10 | 01 | 1 | PDF-05 | V4 | Backend: Employee-Auth-Context → 200 (kein Admin-Gate) | integration | `cargo test -p service_impl pdf_shiftplan::employee_auth_returns_200` | ❌ Wave 0 | ⬜ pending |
| 49-01-11 | 01 | 1 | PDF-05 | V2 | Backend: fehlende Auth → 401 | integration | `cargo test -p service_impl pdf_shiftplan::unauthenticated_returns_401` | ❌ Wave 0 | ⬜ pending |
| 49-02-01 | 02 | 2 | DRY-Refactor | — | Scheduler ruft `PdfShiftplanService::render_week_pdf` mit `Authentication::Full` statt View+SalesPerson+Render direkt | unit | `cargo test -p service_impl pdf_export_scheduler` | ✅ existiert (ANPASSEN) | ⬜ pending |
| 49-03-01 | 03 | 3 | PDF-04 | — | Frontend: pure `should_show_pdf_button(week_status, shiftplan_id: Option<Uuid>) -> bool` — 8 Kombis (4 WeekStatus × Some/None) | unit | `cd shifty-dioxus && cargo test should_show_pdf_button` | ❌ Wave 0 | ⬜ pending |
| 49-03-02 | 03 | 3 | PDF-03 | — | FE: Button rendert wenn Predikat true, URL enthält `selected_shiftplan_id`/`year`/`week` | manual/UAT | siehe Manual-Only-Sektion | — | ⬜ pending |
| 49-04-01 | 04 | 3 | Doku | — | REQUIREMENTS.md PDF-03 umformuliert + Nicht-Ziel gestrichen; ROADMAP.md Goal + SC 3 umformuliert (D-49-15/D-49-16) | source-assert | `grep -c "aktuell im UI selektierte" .planning/REQUIREMENTS.md \|\| exit 1` | ❌ Wave 0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `service/src/pdf_shiftplan.rs` — Trait-Definition (`PdfShiftplanService` + `#[automock]`)
- [ ] `service_impl/src/pdf_shiftplan.rs` — `PdfShiftplanServiceImpl` via `gen_service_impl!`
- [ ] `service_impl/src/test/pdf_shiftplan.rs` — 8 Unit-Tests (Mock-basiert)
- [ ] `service_impl/src/test/mod.rs` — `mod pdf_shiftplan;` registrieren
- [ ] `rest/src/pdf_shiftplan.rs` — Handler + `PdfShiftplanApiDoc`
- [ ] `rest/src/lib.rs` — `mod pdf_shiftplan;`, ApiDoc-Nest, `.nest("/shiftplan", …)` mounten
- [ ] Test-Anpassung: `service_impl/src/test/pdf_export_scheduler.rs::TestDeps` — Mock von `PdfShiftplanService` statt View+SalesPerson-Direct-Calls
- [ ] `shifty-dioxus/src/page/shiftplan.rs` — pure `should_show_pdf_button` + Tests
- [ ] `shifty-dioxus/src/i18n/mod.rs` — `Key::PdfDownload`
- [ ] `shifty-dioxus/src/i18n/{de,en,cs}.rs` — Übersetzung "PDF" (alle 3)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Button-Klick lädt PDF mit korrektem Dateiname im Browser-Download | PDF-03 | Browser-Download-UX + Cookie-Auth-Durchreichung nicht in cargo-Test reproduzierbar | `nix develop -c cargo run` + `cd shifty-dioxus && dx serve`; als Employee einloggen; Shiftplan-Seite; KW mit Status=Planned wählen; Button klicken; Download-Ordner prüfen (`schichtplan-{yyyy}-KW{ww}.pdf`) |
| Button ist unsichtbar bei WeekStatus=Unset/InPlanning | PDF-04 | Reactive Dioxus-Signal-Rerender nicht ohne WASM-Runtime verifizierbar | Von Planned-KW zu InPlanning-KW navigieren; Button muss verschwinden ohne Reload |
| PDF öffnet ohne Fehler im PDF-Reader | PDF-03 | Renderer-Output ist binary; visuelle Nutzbarkeit ist Phase-50-Ziel, hier nur "keine Corruption" | Downloadete PDF öffnen mit Standard-Reader (Zathura/Firefox); keine Warnung, seitenzählbar |

---

## Security Domain

Übernommen aus `49-RESEARCH.md §Security Domain`:

| ASVS Category | Applies | Threat Refs (in PLAN.md `<threat_model>`) |
|---------------|---------|-------------------------------------------|
| V2 Authentication | yes | T-49-01 (fehlende Auth → 401 via `forbid_unauthenticated`-Middleware) |
| V4 Access Control | yes | T-49-02 (Employee-Zugriff nicht durch Admin-Gate blockiert — bewusste Requirement PDF-05) |
| V5 Input Validation | yes | T-49-03 (Path-Param-Typing durch Axum + WeekStatus-Gate) |

**Blocking severity:** high (per `workflow.security_block_on = "high"`).

---

## Validation Sign-Off

- [ ] Alle 14 Tasks haben `<automated>` verify ODER Wave 0 Dependency (siehe Wave 0 Requirements)
- [ ] Sampling continuity: keine 3 aufeinanderfolgenden Tasks ohne automated verify — geprüft: `49-01-XX` durchgehend, `49-02-XX` cargo-Test, `49-03-01` cargo-Test, `49-03-02` + `49-04-01` sind manuelle / source-assert
- [ ] Wave 0 deckt alle MISSING references (10 neue Files/Sections in Wave 0 Requirements)
- [ ] Keine `--watch`-Flags (Cargo default: single-run)
- [ ] Feedback latency < 30s (Quick), < 150s (Full)
- [ ] `nyquist_compliant: true` wird gesetzt nach Wave 0

**Approval:** pending
