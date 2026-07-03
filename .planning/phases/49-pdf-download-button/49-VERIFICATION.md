---
phase: 49-pdf-download-button
verified: 2026-07-03T00:00:00Z
uat_confirmed: 2026-07-03
uat_confirmed_by: user (Simon Goller)
uat_note: "PDF-Download-Roundtrip funktioniert (Klick lädt Datei); Dev-Proxy /shiftplan in Dioxus.toml als hotfix b1bfeab nachgezogen"
status: passed
score: 4/4 must-haves verified (SC1..SC4) + 3/3 UAT-Items PASSED
behavior_unverified: 0
overrides_applied: 0
re_verification:
  previous_status: null
  previous_score: null
  gaps_closed: []
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "Manueller UAT: PDF-Download-Button auf Schichtplan-Seite"
    expected: "Klick auf den PDF-Button lädt die aktuell im UI selektierte KW als Datei `schichtplan-{yyyy}-KW{ww:02}.pdf` in den Browser-Download-Ordner (PDF-03). Button sichtbar nur bei WeekStatus ∈ {Planned, Locked}. Button neben iCal-Button, gleiches Styling."
    why_human: "Browser-Interaktion (Klick auf <a href download>) und tatsächlicher Datei-Landepunkt im Download-Ordner sind nur real im Browser prüfbar. WASM-Rendering des Buttons + Cookie-Auth-Durchreichung ans Backend braucht laufende App."
  - test: "Manueller UAT: Sichtbarkeits-Gate (PDF-04) end-to-end"
    expected: "Wechsel der KW im UI zwischen einem `Unset`/`InPlanning`-Slot und einem `Planned`/`Locked`-Slot lässt den Button erscheinen/verschwinden. Kein Disabled-State, kein Tooltip."
    why_human: "Signal-getriebene Sichtbarkeit ist nur im laufenden WASM-Frontend beobachtbar; die pure Predikat-Fn `should_show_pdf_button` ist getestet, aber die Verdrahtung mit `WEEK_STATUS_STORE` an das RSX-Rendering nicht."
  - test: "Manueller UAT: Employee-Auth erhält 200 (PDF-05)"
    expected: "Mit Employee-Rolle (nicht Admin) auf einem Planned/Locked-KW den Button klicken → PDF wird geliefert, keine 403."
    why_human: "Auth-Context-Durchreichung durch die Middleware-Kette + tatsächlich als Employee eingeloggt zu sein ist am zuverlässigsten manuell zu prüfen. Kein Backend-Integrationstest im Router-Level in dieser Phase (bewusst dokumentiert in rest/src/pdf_shiftplan.rs L163–177)."
---

# Phase 49: pdf-download-button Verification Report

**Phase Goal:** Auf der Schichtplan-Seite gibt es einen PDF-Download-Button, der die aktuell im UI selektierte Kalenderwoche des ausgewählten Shiftplans für jeden authentifizierten User ausliefert — aber nur sichtbar, wenn `week_status ∈ {Planned, Locked}`.

**Verified:** 2026-07-03
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (ROADMAP.md Phase 49 Success Criteria)

| # | Success Criterion | Status | Evidence |
|---|---|---|---|
| SC1 | Neuer REST-Endpoint `GET /shiftplan/{shiftplan_id}/{year}/{week}/pdf` liefert PDF mit Content-Disposition-Filename, auth-required, kein Admin-Gate | ✓ VERIFIED | `rest/src/lib.rs:666` nested at `/shiftplan` + `rest/src/pdf_shiftplan.rs:48` route `"/{shiftplan_id}/{year}/{week}/pdf"`. Handler setzt `Content-Type: application/pdf` und `Content-Disposition: attachment; filename="schichtplan-{JJJJ}-KW{NN:02}.pdf"` via `pdf_response()` (L82–93). Auth: nur `forbid_unauthenticated` Middleware (`rest/src/lib.rs:707`), kein Admin-Check im Handler oder Router. Tests: `pdf_response_sets_pdf_content_type_and_filename` + `..._leading_zero_for_single_digit_weeks` + `..._handles_week_52` alle grün (Content-Disposition-Format verifiziert für W03, W27, W52). |
| SC2 | Backend HTTP 409 bei `week_status ∈ {Unset, InPlanning}` | ✓ VERIFIED | Zwei-Ebenen-Gate: (a) REST-Handler Pre-Check `rest/src/pdf_shiftplan.rs:122–137` ruft `week_status_service.get_week_status()` VOR dem Rendering, bei nicht-releasablem Status → `not_releasable_response()` (409 + JSON-Body `{"error":"week-not-releasable"}`). (b) Service-Gate in `service_impl/src/pdf_shiftplan.rs:114–120` mapped ebenfalls auf `ServiceError::ValidationError` als Defense-in-Depth. Tests: `week_status_planned_allows_download`, `week_status_locked_allows_download`, `week_status_unset_blocks_download`, `week_status_in_planning_blocks_download` + `not_releasable_returns_409_json_with_stable_error_code` alle grün. Service-Tests: `week_status_unset_returns_validation_error`, `week_status_in_planning_returns_validation_error`, `week_status_locked_returns_bytes`, `happy_path_returns_bytes` (Planned) alle grün. |
| SC3 | Frontend-Button neben iCal-Button, nur sichtbar bei `week_status ∈ {Planned, Locked}`, kein disabled, kein Tooltip, kein Toast; lädt aktuell selektierte KW; i18n in de/en/cs | ⚠️ PRESENT_BEHAVIOR_UNVERIFIED (routes to human_verification) | Anchor gerendert in `shifty-dioxus/src/page/shiftplan.rs:1174–1181` DIREKT neben dem iCal-Anchor (L1148–1155). Identische Styling-Klassen `"px-3 py-1.5 rounded-md text-body font-medium border bg-surface text-ink border-border-strong inline-flex items-center gap-1 hover:bg-surface-alt"`. Icon-Prefix `↓` im mono-Span, Label via `i18n.t(Key::PdfDownload)`. URL: `format!("{}/shiftplan/{}/{}/{}/pdf", backend_url_pdf, sp_id, y, w)` mit `y = *year.read()`, `w = *week.read()` — selektierte KW, nicht heute. Sichtbarkeit gate `if should_show_pdf_button(ws, sp_id_opt)`. Reine Predikat-Fn `should_show_pdf_button(status, shiftplan_id)` in `shifty-dioxus/src/page/shiftplan.rs:95–97`. Test-Matrix vollständig (4×2 = 8 Tests): `some_id_planned`, `some_id_locked`, `some_id_unset`, `some_id_in_planning`, `none_id_planned`, `none_id_locked`, `none_id_unset`, `none_id_in_planning` alle grün. i18n `Key::PdfDownload` deklariert in `shifty-dioxus/src/i18n/mod.rs:88`; Übersetzung `"PDF"` in allen drei Locales: `de.rs:44`, `en.rs:44`, `cs.rs:44`. Verbleibt behavior-unverified, weil tatsächliches Rendering + Klick-Download-Roundtrip nur im Browser beobachtbar ist → Human-UAT (siehe unten). |
| SC4 | DRY: `PdfShiftplanService` kapselt View + SalesPerson + WeekStatus + pdf_render; `PdfExportScheduler` konsumiert diesen Service statt inline zu orchestrieren | ✓ VERIFIED | Service-Trait `service/src/pdf_shiftplan.rs:36–60` definiert `render_week_pdf()`. Impl `service_impl/src/pdf_shiftplan.rs:96–143` orchestriert genau die vier Steps (1. WeekStatus-Gate, 2. View, 3. SalesPerson + `filter_active`, 4. `pdf_render::render_shiftplan_week_pdf`). Scheduler `service_impl/src/pdf_export_scheduler.rs:365–381` ruft `self.pdf_shiftplan_service.render_week_pdf(shiftplan_id, y, w, Authentication::Full, None)` — NO direct calls left (grep `shiftplan_view_service|sales_person_service|render_shiftplan_week_pdf` in `pdf_export_scheduler.rs` returns zero matches). DI-Reihenfolge in `shifty_bin/src/main.rs:1246–1266`: `pdf_shiftplan_service` konstruiert VOR `pdf_export_scheduler`, konsumiert die drei Basic-Services. Scheduler-Test `scheduler_calls_pdf_shiftplan_service_with_full_auth` grün. |

**Score:** 4/4 SCs verifiziert (SC3 = present-behavior-unverified: Code wired, Browser-Klick-Roundtrip pending).

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `service/src/pdf_shiftplan.rs` | Trait `PdfShiftplanService` + `filename_for` helper | ✓ VERIFIED | 70 Zeilen. `render_week_pdf`-Trait-Methode + `#[automock]`. `filename_for(year, week)` in service-Crate → REST + service_impl teilen sich ihn. |
| `service_impl/src/pdf_shiftplan.rs` | Impl mit 4-Step-Assemble + filter_active helper | ✓ VERIFIED | 149 Zeilen. `gen_service_impl!`-Deps auf ViewService/SalesPerson/WeekStatus/Permission/TransactionDao. Impl `render_week_pdf` folgt der Reihenfolge im Modul-Doc. Re-export `filename_for` L148. |
| `service_impl/src/test/pdf_shiftplan.rs` | Mock-basierte Unit-Tests (Happy-Path, Status-Matrix, Filter, Context-Weitergabe, Error-Bubble) | ✓ VERIFIED | 10 async Tests + 1 sync Filename-Test = 11 total. Alle 12 (inkl. Scheduler-Test) grün via `cargo test --workspace pdf_shiftplan`. |
| `rest/src/pdf_shiftplan.rs` | REST-Handler + PdfShiftplanApiDoc + 3 Kernstück-Tests | ✓ VERIFIED | 276 Zeilen. Router `generate_route()`, Handler `download_week_pdf`, `PdfShiftplanApiDoc`. 8 Unit-Tests (4 Status-Matrix + 1×409-Body + 3×200-Filename-Format). |
| `rest/src/lib.rs` | Registrierung Modul + ApiDoc-Nest + Router-Nest | ✓ VERIFIED | L22 `mod pdf_shiftplan`, L436 assoc-type `PdfShiftplanService`, L478 Accessor `pdf_shiftplan_service()`, L593 ApiDoc-Nest `(path = "/shiftplan", ...)`, L666 Router-Nest `.nest("/shiftplan", pdf_shiftplan::generate_route())`. |
| `service_impl/src/pdf_export_scheduler.rs` | Refactored: delegiert an PdfShiftplanService | ✓ VERIFIED | Modul-Doc L26–35 dokumentiert Refactor. Deps L71 `pdf_shiftplan_service`. Assemble in `run_once_now` L358–381 ruft nur noch `pdf_shiftplan_service.render_week_pdf`. Keine restlichen direkten View-/SalesPerson-/pdf_render-Aufrufe. Filename via `crate::pdf_shiftplan::filename_for` L383. |
| `shifty_bin/src/main.rs` | DI-Wiring `PdfShiftplanServiceDependencies` + Scheduler-Dep | ✓ VERIFIED | L324–326 assoc-type, L340–351 Deps + Type-Alias, L697 Feld im Kontainer, L735 Assoc-Type-Impl, L845–847 Accessor, L1246–1256 Konstruktion vor Scheduler, L1258–1266 Scheduler bekommt Service als Dep, L1321 Injection. |
| `shifty-dioxus/src/page/shiftplan.rs` | should_show_pdf_button + Anchor neben iCal + 8-case Tests | ✓ VERIFIED | Predikat L95–97 pure Fn. Anchor RSX L1164–1185 direkt nach iCal-Block. Test-Modul `pdf_button_tests` L2141–2193 mit exakt 8 Test-Cases (4×2 Matrix). |
| `shifty-dioxus/src/i18n/{mod.rs,de.rs,en.rs,cs.rs}` | `Key::PdfDownload = "PDF"` in allen drei Locales | ✓ VERIFIED | `mod.rs:88` deklariert, `de.rs:44`/`en.rs:44`/`cs.rs:44` je `"PDF"` — konsistent mit D-49-14. |

### Key Link Verification

| From | To | Via | Status | Details |
|---|---|---|---|---|
| `rest/src/pdf_shiftplan.rs::download_week_pdf` | `rest_state.pdf_shiftplan_service().render_week_pdf(...)` | State-Accessor + Trait-Call | ✓ WIRED | L142–144 im Handler. |
| `rest/src/pdf_shiftplan.rs::download_week_pdf` | `rest_state.week_status_service().get_week_status(...)` | Pre-Check | ✓ WIRED | L124–128, VOR dem Service-Call. |
| `service_impl/src/pdf_shiftplan.rs::render_week_pdf` | `pdf_render::render_shiftplan_week_pdf(...)` | direkter Aufruf | ✓ WIRED | L136–141. |
| `service_impl/src/pdf_export_scheduler.rs::run_once_now` | `self.pdf_shiftplan_service.render_week_pdf(...)` | Dep-Aufruf statt inline | ✓ WIRED (DRY) | L365–367. Keine Restaufrufe an View/SalesPerson/pdf_render. |
| `shifty-dioxus/src/page/shiftplan.rs (RSX)` | `should_show_pdf_button(ws, sp_id_opt)` | Predikat-Gate | ✓ WIRED | L1172. |
| `shifty-dioxus/src/page/shiftplan.rs (RSX)` | Backend `/shiftplan/{id}/{y}/{w}/pdf` | `<a href download>` | ✓ WIRED (structure) | L1174–1181; Cookie-Auth durchgereicht. Manueller Klick-Roundtrip Human-UAT. |
| `shifty_bin/src/main.rs` | `PdfShiftplanServiceImpl::new(view, sp, ws, perm, tx)` | Konstruktion | ✓ WIRED | L1246–1256, DI-Order korrekt. |
| `shifty_bin/src/main.rs` | `PdfExportSchedulerService::new(..., pdf_shiftplan_service, ...)` | Dep-Injection | ✓ WIRED | L1258–1266. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|---|---|---|---|---|
| REST-Handler `download_week_pdf` | `bytes: Vec<u8>` | `pdf_shiftplan_service.render_week_pdf(...)` → `pdf_render::render_shiftplan_week_pdf(...)` (real render, kein Static) | Yes | ✓ FLOWING |
| RSX `<a>`-Anchor | `href` | `format!("{}/shiftplan/{}/{}/{}/pdf", backend_url_pdf, sp_id, y, w)` mit realen Signals `selected_shiftplan_id`, `year`, `week` | Yes | ✓ FLOWING |
| RSX `<a>`-Anchor | `pdf_label` | `i18n.t(Key::PdfDownload)` → real Locale-Lookup | Yes | ✓ FLOWING |
| Scheduler | `bytes` per Week | `pdf_shiftplan_service.render_week_pdf(shiftplan_id, y, w, Authentication::Full, None)` | Yes | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| Backend workspace tests grün | `cargo test --workspace` | 629 + 64 + 24 + 13 + 12 + 11 + 11 + 8 + 5 + 5 + 3 + 2 = 787+ Tests, alle grün | ✓ PASS |
| Backend Clippy sauber (Gate) | `cargo clippy --workspace -- -D warnings` | Exit 0 | ✓ PASS |
| PdfShiftplanService Unit-Tests | `cargo test --workspace pdf_shiftplan` | 12 passed | ✓ PASS |
| REST-Handler Unit-Tests | `cargo test -p rest pdf_shiftplan` | 8 passed (Content-Type/Content-Disposition/Body-Format) | ✓ PASS |
| FE Predikat-Test-Matrix | `cd shifty-dioxus && cargo test pdf_button` | 8 passed (4×2 Matrix) | ✓ PASS |
| Filename-Format `schichtplan-{yyyy}-KW{ww:02}.pdf` inkl. zero-pad + week 52 | `cargo test content_disposition_filename_format_helper` + `pdf_response_filename_uses_leading_zero_for_single_digit_weeks` + `pdf_response_filename_handles_week_52` | 3 passed | ✓ PASS |
| Klick auf PDF-Button lädt Datei im Browser | Manueller Browser-Test mit realem Session-Cookie | pending | ? SKIP (Human-UAT) |

### Probe Execution

Keine phasenspezifischen probes deklariert (weder in PLAN.md noch in SUMMARY.md). Konventionelles `scripts/*/tests/probe-*.sh` existiert im Repo nicht für Phase 49 (Grep leer). SKIPPED — nicht anwendbar.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|---|---|---|---|---|
| PDF-03 | 49-01, 49-02, 49-04 | Download-Button auf Schichtplan-Seite; Klick lädt aktuell im UI selektierte KW; Filename `schichtplan-{JJJJ}-KW{NN}.pdf` | ✓ SATISFIED (Code) / ? NEEDS HUMAN (Klick-Roundtrip) | Backend liefert Filename via `pdf_response()` mit `filename_for()` (zero-pad, `KW03/KW27/KW52`-Tests grün). FE-Anchor konstruiert URL aus `year`/`week`-Signals, Download-Attribut gesetzt. Verifikation im Browser = Human-UAT (siehe Section unten). |
| PDF-04 | 49-01, 49-02, 49-04 | Button-Sichtbarkeit gated auf `week_status ∈ {Planned, Locked}`; Backend gibt 409 bei `Unset/InPlanning` | ✓ SATISFIED | FE: `should_show_pdf_button` mit 8-case Test-Matrix. BE: 409-Pre-Check im Handler + `ValidationError` im Service, Status-Matrix-Tests komplett auf beiden Ebenen. |
| PDF-05 | 49-01, 49-02, 49-04 | Kein Admin-Gate; alle authentifizierten User (inkl. Employee); keine Sales-Person-Filterung (ganzes KW-PDF sichtbar) | ✓ SATISFIED (Code) / ? NEEDS HUMAN (Employee-Auth-Klick) | Route in Auth-Layer, aber kein Admin-Check im Handler (`rest/src/pdf_shiftplan.rs:113–149`). Service filtert nur `deleted.is_none()`, KEINE per-User-Filterung (`service_impl/src/pdf_shiftplan.rs:87–93 filter_active`). Test `service_render_does_not_leak_deleted_sales_persons` verifiziert Filter. Employee-Auth-200-Prüfung bewusst als Human-UAT dokumentiert (`rest/src/pdf_shiftplan.rs:163–177` — Router-Level-Integrationstest wurde als unverhältnismäßig groß befunden gegen `RestStateDef` mit 37 assoc-types). |

Keine ORPHANED Requirements (Phase 49 in ROADMAP nennt exakt PDF-03/04/05, alle drei sind in Plan-Frontmatter deklariert und behandelt).

### Anti-Patterns Found

Grep nach `TBD|FIXME|XXX|TODO|HACK|PLACEHOLDER` in den in dieser Phase modifizierten Dateien:

| File | Line | Pattern | Severity | Impact |
|---|---|---|---|---|
| — | — | — | — | Keine Blocker-Marker gefunden. Ein `TODO` in `pdf_export_scheduler.rs` L34 (Modul-Doc) ist rein informativ ("kein leaky Export"), kein Debt-Marker. Grep in `service/src/pdf_shiftplan.rs`, `service_impl/src/pdf_shiftplan.rs`, `service_impl/src/test/pdf_shiftplan.rs`, `rest/src/pdf_shiftplan.rs`, `shifty-dioxus/src/page/shiftplan.rs` (PDF-Sektion), i18n-Dateien → 0 blockende Marker. |

Keine Stubs entdeckt: `filter_active` ist eine echte Filter-Fn, nicht `return vec![]`. Handler ruft echten Service (nicht static-return). RSX-Render ist an echte Signals gebunden.

### Human Verification Required

Siehe frontmatter `human_verification:`.

Zusammenfassung:

1. **PDF-Download-Button Klick-Roundtrip** — Datei landet mit korrektem Namen im Browser-Download.
2. **Sichtbarkeits-Gate im Live-UI** — Wechsel zwischen `Planned/Locked` und `Unset/InPlanning` KWs.
3. **Employee-Auth erhält 200** — Nicht-Admin-Kontext im laufenden System, keine 403 vom Backend.

### Gaps Summary

Keine Gaps. Alle vier SCs sind entweder verifiziert (SC1, SC2, SC4) oder present-behavior-unverified mit exakt umrissenem Human-UAT-Pfad (SC3). Alle 16 D-49-Decisions haben ein zugehöriges Code-Artefakt und sind konsistent gewired. Backend-Tests (792 total, alle grün) + FE-Tests (8/8 grün) + Clippy-Gate (sauber) sichern die statisch prüfbaren Anteile. Die verbleibenden Prüfungen sind reine Browser-Interaktions-/Auth-Flow-Tests.

---

_Verifiziert: 2026-07-03_
_Verifier: Claude (gsd-verifier), goal-backward auf Phase 49_
