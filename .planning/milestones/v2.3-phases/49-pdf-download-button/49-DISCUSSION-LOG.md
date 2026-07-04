# Phase 49: pdf-download-button - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 49-CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-03
**Phase:** 49-pdf-download-button
**Areas discussed:** G1 (Endpoint-Shape), G2 (Backend-Wiring/DRY), G3 (FE-Download-Mechanik), G4 (FE-Status-Enable-Quelle), G5 (Button-Platzierung + Fehler-UX)
**Mode:** discuss, Textform (per feedback_prefer_text_questions Memory)

---

## G1 — Endpoint-Shape & REST-Modul-Verortung

| Option | Description | Selected |
|--------|-------------|----------|
| (a) `GET /shiftplan/{shiftplan_id}/{year}/{week}/pdf` | `shiftplan_id` explizit im Pfad | ✓ |
| (b) `GET /shiftplan/{year}/{week}/pdf` mit Query-Param für shiftplan_id | Roadmap-Wortlaut, id als `?shiftplan_id=…` | |
| (c) Hardcoded „ersten im Catalog" wie Scheduler heute | Kein id-Handling im Handler | |

**User's choice:** (a)
**Notes:** Präferenz für expliziten, self-documenting URL-Pfad ohne Query-Param-Magie. Passt zu bestehender Convention `/sales-person/{id}/ical`. REST-Modul: eigenes `pdf_shiftplan.rs` (Claude's Discretion — analog `pdf_export_config`).

---

## G2 — Backend-Wiring / DRY-Auflösung

| Option | Description | Selected |
|--------|-------------|----------|
| (A) Neuer Business-Logic-Service `PdfShiftplanService` (Trait + `gen_service_impl!`) | Volle DRY-Auflösung inkl. Scheduler-Refactor; Trait-Objekte für Mock-Tests | ✓ |
| (A') Free-standing `async fn assemble_week_pdf(view_svc, sp_svc, id, y, w, ctx)` in `pdf_render` | DRY ohne Trait, weniger DI-Ceremony | |
| (B) Handler orchestriert direkt (3 Services + `pdf_render`) | Duplikat zum Scheduler-Body — nicht DRY | |
| (C) Neue Public-Trait-Methode auf `PdfExportScheduler` | Bläht Cron-Service auf, mischt Concerns | |

**User's choice:** (A)
**Notes:** User bestand explizit auf DRY („Es muss doch bereits einen Service geben, der das PDF generiert. Ansonsten halten wir uns nicht an DRY."). Klarstellung im Chat: Renderer selbst (`pdf_render::render_shiftplan_week_pdf`) ist bereits DRY; was fehlt, ist DRY auf der Orchestrations-Ebene (View + SalesPerson-Filter + Render). Entscheidung für echten Service statt Free-Standing-Fn wegen Testbarkeit via `#[automock]`. Scheduler-Refactor (D-49-08) ist verbindlicher Bestandteil dieser Phase, sonst ist die DRY-Auflösung nur halb.

---

## G3 — FE-Download-Mechanik

| Option | Description | Selected |
|--------|-------------|----------|
| (a) `<a href="…/pdf" download="…">` | Cookie-Auth über Browser, kein WASM-Fetch | ✓ |
| (b) `reqwest::get(…).bytes()` → `Blob` → `createObjectURL` → anchor-click | WASM, mehr Kontrolle über Loading/Fehler | |

**User's choice:** (a)
**Notes:** Analog zum bestehenden iCal-Button (`shifty-dioxus/src/page/shiftplan.rs:1130–1138`). User ergänzte in derselben Antwort: „Aber nur anzeigen, wenn die KW Status geplant oder gesperrt ist" — d.h. Button ist versteckt statt disabled, siehe G4.

---

## G4 — FE-Enable-Quelle (welche KW entscheidet über Sichtbarkeit + Ziel-KW?)

| Option | Description | Selected |
|--------|-------------|----------|
| (i) Sichtbarkeit + Ziel-KW = heute-KW; navigierte KW ignoriert | Requirement-konform (PDF-03 original) | |
| (ii) Sichtbarkeit + Ziel-KW = im UI selektierte KW | Deviation zu PDF-03; einfacher (nutzt bestehenden `WEEK_STATUS_STORE`) | ✓ |
| (iii) Button immer sichtbar, Backend-409 als einziger Gate | Verletzt PDF-04 | |

**User's choice:** (ii) — mit expliziter Änderung der Antwort in Runde 2
**Notes:** Erste Antwort war mehrdeutig („Button sollte nur angezeigt werden, je nach Status"). Nachfrage klarstellte: User wollte (ii), also selektierte KW entscheidet UND wird geladen. Das ist eine **bewusste Requirement-Deviation** von PDF-03 („aktuelle Kalenderwoche basierend auf heute") und dem Nicht-Ziel „Wochenwahl über die UI-Navigation". User gab Freigabe für REQUIREMENTS.md + ROADMAP-Update im selben Commit („Ja" auf die Rückfrage). Vorteil: FE-Implementation nutzt bestehende Signals ohne Zusatz-Fetch für heute-KW; UX ist konsistenter (Klick lädt genau, was der User gerade sieht).

---

## G5 — Button-Platzierung & Fehler-UX

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Neben iCal-Button (Zeile ~1140 in shiftplan.rs) | Symmetrie mit bestehendem Download-Muster | ✓ |
| (b) Im Header-Bereich neben WeekStatusBadge | Anderes visuelles Cluster | |
| (c) Im DayButtonBar | Zu weit weg vom KW-Context | |
| (α) Fehler-Toast bei 409 | UI-Reaktion auf Race-Case | |
| (β) Inline-Banner bei 409 | Analog zu feedback_warnings_inline_not_dialog | |
| (γ) Keine Fehler-UI (Button versteckt = kein Fehler möglich) | Race-Case ist Rand-Fall | ✓ |

**User's choice:** (a) + (γ)
**Notes:** „Der soll neben dem iCal button sitzen. Eine Warnung braucht man nicht." — konsequente Reduktion. Kein Tooltip nötig, weil kein disabled-Zustand existiert. i18n schrumpft auf einen Key (`PdfDownload`).

---

## Claude's Discretion

- **Naming:** `PdfShiftplanService` vs. `PdfDownloadService` — Planner entscheidet nach REST-Kontext-Lesbarkeit.
- **Test-Struktur:** Unit-Tests via `#[automock]` für alle drei konsumierten Services; Integrationstest für die 200/409/401-Matrix; FE-Test als reine Predikat-Fn `should_show_pdf_button(week_status, shiftplan_id) -> bool`.
- **Scheduler-Refactor-Umfang:** Nach dem Refactor darf `PdfExportSchedulerDeps` `shiftplan_view_service`- und `sales_person_service`-Deps streichen, wenn sie nur noch via `PdfShiftplanService` erreicht werden — Planner nach `grep` entscheiden.
- **OpenAPI-Response-Body:** `content = "application/pdf"` mit `Vec<u8>`-Body-Schema, 409/401/404-Responses annotieren.
- **409-Body-Format:** JSON vs. text/plain — Planner konform zu bestehendem `ServiceError`-Mapping.
- **Router-Kollision:** Nest `/shiftplan` vs. `/shiftplan-pdf` — Planner sanity-check gegen bestehende `.nest`-Aufrufe in `rest/src/lib.rs`.
- **Icon-Prefix + Label-Länge:** `↓ PDF` als Kurz-Label; User kann später auf „PDF herunterladen" erweitern, wenn gewünscht.

## Deferred Ideas

- **Multi-Week-Batch-Download** — Single-Week reicht für v2.3.
- **PDF-Preview im Browser** statt Download — nicht diskutiert.
- **Personal-PDF (nur eigene Bookings)** — explizit Nicht-Ziel in REQUIREMENTS.md (PDF-05).
- **Fehler-Toast/Banner** — bewusst weggelassen, trivialer Nachtrag möglich.
- **Loading-Spinner** — beim `<a href>`-Ansatz unnötig (Browser-Standard-Download-UI).

## Requirement-Deviation Audit

- **PDF-03** — „aktuelle KW basierend auf heute" → „aktuell im UI selektierte KW"
- **Nicht-Ziel „Wochenwahl über die UI-Navigation"** — gestrichen (bewusst umgekehrt)
- **PDF-04** — „disabled + Tooltip" → „versteckt (kein disabled-Zustand)"; Backend-409-Bedingung von {`None`, `Planning`} auf {`Unset`, `InPlanning`} umbenannt (nur Terminologie-Anpassung an tatsächliches `WeekStatus`-Enum)
- **ROADMAP Phase 49 Goal + SC 3** — sinngemäß angeglichen; neue SC 4 dokumentiert den `PdfShiftplanService` als DRY-Fix inkl. Scheduler-Refactor

Diese Edits werden im selben Commit wie CONTEXT.md + DISCUSSION-LOG.md aufgenommen (`docs(49): capture phase context`).
