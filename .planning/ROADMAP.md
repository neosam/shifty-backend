# Roadmap: Shifty Backend

## Milestones

- ✅ **v1.0 Range-Based Absence Management** — Phasen 1–4 (shipped 2026-05-03) — siehe [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Slot Capacity & Constraints** — Phase 5 (shipped 2026-05-04) — siehe [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)
- ✅ **v1.2 Frontend rest-types Konsolidierung** — Phasen 6–7 (shipped 2026-05-07) — siehe [`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md)
- ◆ **v1.3 Frontend Abwesenheiten + UI-Closure-Restanten** — Phasen 8 (✓), 8.1, 8.2, 9–13 (active, started 2026-05-07)

## Phases

### v1.3 Frontend Abwesenheiten + UI-Closure-Restanten (active)

- [x] **Phase 8: Absence-CRUD-Page Foundation** (Frontend) — completed 2026-05-08, 9 plans, code-side fertig, int-UAT deferred zu Phase 8.1
  Neue Top-Level-Route `absences` mit CRUD gegen `/absence-period`, HR/Employee-Sicht aus Auth-Context, Form (Range-Picker + Kategorie + Description) und Forward-Warnings-Anzeige.
  Requirements: FUI-A-01, FUI-A-02, FUI-A-03, FUI-A-04
  Success Criteria:
  1. Route `/absences` ist via Menü erreichbar; HR-Privileg-Check schaltet Filter über alle Mitarbeiter frei (Auth-Context, kein User-Toggle)
  2. Form erlaubt CRUD eines `AbsencePeriodTO` mit Datum-Range-Picker (Ganztage), Kategorie-Dropdown (`Vacation`/`SickLeave`/`UnpaidLeave`), Description; Self-Overlap-`422` wird als Validation-Error gerendert
  3. `AbsencePeriodCreateResultTO.warnings[]` aus POST/PUT-Antwort wird als nicht-blockierende Hinweisliste angezeigt
  4. `cargo build --target wasm32-unknown-unknown` grün; UAT-Smoke gegen Integrationsumgebung (HR + Employee Login je einmal Anlage + Edit + Delete) — **deferred, siehe 08-HUMAN-UAT.md + Phase 8.1**

- [ ] **Phase 8.1: Cutover-Migration-UI** (Frontend, Closure-Phase für Phase 8)
  Admin-UI für die `extra_hours` → `absence_period`-Datenmigration. 3-Stage-Wizard (Profile → Dry-Run → Commit) mit Drift-Resolution-Liste, Per-Eintrag-Aktionen (Delete / Edit / Convert-to-Range / Skip) und Bulk-Aktionen. Schließt den Phase-8-int-UAT-Block, der durch reale Buchungs-Pattern-Diversität entstanden ist (Auto-Heuristik in Plan 08-09 deckt nicht alle Patterns ab — siehe 08-HUMAN-UAT.md gap-1).
  Requirements: (Closure-Phase, kein neues FUI-Requirement; löst Phase-8-Adoption-Block)
  Success Criteria:
  1. Admin-Route mit `cutover_admin`-Privileg-Gate; 3 sichtbare Stages; Profile + Dry-Run liefern strukturiertes Ergebnis-Display (Quarantine-Counts, Carryover-Diff)
  2. Drift-Resolution-Liste rendert pro `quarantined_entry` Datum/Wochentag/Stunden/Reason-Text/Suggested-Action (alles aus inline `gate_drift_report` aus Plan 08-08); Per-Eintrag-Aktionen für Delete / Edit-extra_hours / Convert-to-AbsencePeriod / Skip
  3. Bulk-Aktion "Alle Wochenpauschalen für (sales_person, category, year) konvertieren" verfügbar
  4. Cutover-Commit erst aktiv wenn `quarantined_rows == 0`; Confirmation-Dialog vor destruktivem Commit; Idempotenz-Hinweis nach Abschluss
  5. Optional: Feiertag-Konsistenz-Fix in `detect_weekly_lump_sum` (Plan 08-09 Inkonsistenz mit `derive_hours_for_range` — Group D Drifts wie Sonja Vac 2026)
  6. Phase-8-HUMAN-UAT (35 Schritte) wird auf int durchlaufen und gemeinsam mit Phase 8.1 closed; gap-1 in 08-HUMAN-UAT.md auf `resolved` gesetzt

- [x] **Phase 8.2: Manual-Range-Convert für Quarantäne** (Backend + Frontend, gap-1a-Closure) — completed 2026-05-10
  Erweitert die Cutover-UI um manuelles Konvertieren: Wenn die Heuristik einen Quarantäne-Eintrag nicht auflösen kann (Karin-Pattern, gap-1a — Vertragswechsel mit differing `hours_per_day` mid-week), gibt der Admin/HR den `absence_period`-Zeitraum (start/end) selbst vor. Backend erweitert `convert_quarantine_entry` um optionales `manual_range`, skipt die Heuristik und schreibt direkt. Frontend ersetzt das stub-bleibende `EditExtraHoursModal` durch ein `ManualConvertModal` mit Date-Range-Picker.
  Requirements: (Closure-Phase, schließt gap-1a aus 08.1-10-SUMMARY)
  Success Criteria:
  1. `CutoverConvertQuarantineEntryPayload.manual_range: Option<{ start, end }>` neu; bei `Some` skipt die Heuristik und nutzt den gegebenen Range; gleicher `synthetic_run_id`-Pfad wie heuristischer Convert
  2. Per-Eintrag-Modal in der Cutover-Page: Date-Range-Picker (von / bis), Category read-only, Submit dispatcht `CutoverAction::ManualConvert` und liefert `refreshed_drift_report` inline (D-08-Pattern aus 8.1)
  3. Karin-Diagnose-Test (`diagnose_int_drift_pattern_karin_*`) plus 1 neuer Test: manual_range löst die Karin-Quarantäne ohne Backend-Heuristik-Anpassung
  4. WASM-Build + Backend cargo test workspace grün; Privilege bleibt `cutover_admin OR hr` (D-23 aus 8.1)

- [ ] **Phase 9: Booking-Flow Reverse-Warnings + Copy-Week** (Frontend)
  Shiftplan-Editor-Buchungen laufen über `POST /shiftplan-edit/booking` mit Reverse-Warnings-Confirm-Dialog; Wochen-Kopie über `POST /shiftplan-edit/copy-week` mit aggregierten Warnings.
  Requirements: FUI-A-05, FUI-A-06
  Success Criteria:
  1. Booking aus Shiftplan-Editor postet auf `/shiftplan-edit/booking`; `BookingCreateResultTO.warnings[]` löst Dioxus-Confirm-Dialog aus (kein `window.confirm`) vor finaler Buchung
  2. Wochen-Kopie postet auf `/shiftplan-edit/copy-week`; aggregierte `CopyWeekResultTO.warnings[]` werden in einer zusammengefassten Anzeige gerendert
  3. Alter `POST /booking` bleibt parallel verfügbar (verifiziert durch grep-Check, dass alte Call-Sites unverändert sind)

- [ ] **Phase 10: Shiftplan-View Unavailability-Marker** (Frontend)
  Shiftplan-Wochen-View visualisiert `UnavailabilityMarkerTO` farbig pro Tag pro Person mit drei Visual-States.
  Requirements: FUI-A-07
  Success Criteria:
  1. Wochen-View nutzt per-sales-person Endpoint `/shiftplan-info/{shiftplan_id}/{year}/{week}/sales-person/{sales_person_id}`
  2. `UnavailabilityMarkerTO::AbsencePeriod` mit Kategorie-Farbe gerendert (Vacation = grün, SickLeave = orange, UnpaidLeave = grau — Final-Farben in UI-SPEC)
  3. `UnavailabilityMarkerTO::ManualUnavailable` neutral gerendert; `UnavailabilityMarkerTO::Both` mit eigener Visual-Indication (signalisiert redundanten manuellen Eintrag nach Cutover, optional Aufräum-Button)

- [ ] **Phase 11: Migrations-Hinweis-UX + Deprecation-Handling** (Frontend)
  Alte `extra_hours`-basierten "Urlaub eintragen"-Eingangswege werden auf neue Maske umgelenkt; nach Cutover wird `403 ExtraHoursCategoryDeprecatedErrorTO` mit User-Hinweis abgefangen.
  Requirements: FUI-A-08
  Success Criteria:
  1. `add_extra_hours_form.rs`, `extra_hours_modal.rs`, `add_extra_days_form.rs`, `add_extra_hours_choice.rs` verlinken für `Vacation` / `SickLeave` / `UnpaidLeave` auf `/absences` (Soft-Migration vor Cutover)
  2. `403 ExtraHoursCategoryDeprecatedErrorTO`-Response wird abgefangen, Toast/Banner mit Migrations-Hinweis und Link zur neuen Maske gerendert
  3. Cutover-Flag-Status (`absence_range_source_active`) wird defensiv gehandhabt: lesen immer aus `/absence-period`; Schreiben über alte Maske nur falls Flag noch aus, sonst Redirect

- [ ] **Phase 12: UI-Closure v1.1/v1.2-Restanten** (Frontend)
  Schließe sichtbares `current_paid_count`/`max_paid_employees`-Rendering, Capacity-Editor in Slot-Settings, sichtbare `VolunteerWork`/`UnpaidLeave`-Kategorien und `cap_planned_hours_to_expected`-Settings.
  Requirements: FUI-01, FUI-02, FUI-03, FUI-04
  Success Criteria:
  1. `current_paid_count` ist im Shiftplan-Week-View pro Slot sichtbar; mit Layout-Variante `2/3 bezahlt` wenn `max_paid_employees` konfiguriert; `Warning::PaidEmployeeLimitExceeded` wird visuell hervorgehoben
  2. Slot-Settings haben Capacity-Editor mit Clear-Button für `None` (kein Limit); Round-Trip-Test (open → save unverändert) bewahrt den Backend-Wert
  3. `VolunteerWork` / `UnpaidLeave` werden in Extra-Hours-Listen sichtbar gerendert (kein `rsx! { "" }` mehr aus v1.2 Plan 06-04); Kategorien sind in der Anlage-Form auswählbar (sofern Cutover-Flag-Konsistenz erlaubt)
  4. `cap_planned_hours_to_expected` ist im Sales-Person-Settings-UI als Toggle editierbar; Server-Round-Trip verifiziert

- [ ] **Phase 13: i18n-Vollständigkeits-Audit + v1.3 Smoke-Closure** (Subsumption-Closure)
  Cross-Phase i18n-Audit: alle in v1.3 hinzugefügten Strings sind in De / En / Cs vollständig. Plus Final-UAT auf Integrationsumgebung (Subsumption-Pattern wie v1.2 Phase 7).
  Requirements: FUI-A-09
  Success Criteria:
  1. Alle in Phasen 8–12 hinzugefügten i18n-Keys sind in `en.rs`, `de.rs`, `cs.rs` vollständig (kein Locale::En-statt-Locale::De-Bug); diff-Audit dokumentiert
  2. Final-UAT: HR-Login + Employee-Login je einmal durch alle drei Locales (Page-Load, Form-Anlage, Warning-Render, Deprecation-Toast)
  3. Backend `cargo check --workspace` + `cargo test --workspace` re-verifiziert (keine Regression durch Frontend-Phasen-Coupling)
  4. WASM-Build `cargo build --target wasm32-unknown-unknown` grün als finaler Compile-Gate

<details>
<summary>✅ v1.0 Range-Based Absence Management (Phasen 1–4) — SHIPPED 2026-05-03</summary>

- [x] **Phase 1: Absence Domain Foundation** (5/5 plans) — completed 2026-05-01
  Neue parallele `absence` Domain (DAO + Service + REST + Permission), additiv, ohne Reporting-Wirkung
- [x] **Phase 2: Reporting Integration & Snapshot Versioning** (4/4 plans) — completed 2026-05-02
  `derive_hours_for_range` + Reporting-Switch hinter Feature-Flag, `CURRENT_SNAPSHOT_SCHEMA_VERSION` 2 → 3 im selben Commit
- [x] **Phase 3: Booking & Shift-Plan Konflikt-Integration** (6/6 plans) — completed 2026-05-02
  Forward/Reverse Booking-Warnings + Shift-Plan-Anzeige aus AbsencePeriod ohne Doppel-Eintragung
- [x] **Phase 4: Migration & Cutover** (8/8 plans) — completed 2026-05-03
  Heuristik-Migration, Validierungs-Gate (< 0.01h Drift-Toleranz), atomarer Feature-Flag-Flip mit Carryover-Refresh, REST-Deprecation. Plus Bonus-Recovery von `extra_hours.update` mit logical_id-Versionierung.

**Full milestone archive:** [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)

</details>

<details>
<summary>✅ v1.1 Slot Capacity & Constraints (Phase 5) — SHIPPED 2026-05-04</summary>

- [x] **Phase 5: Slot Paid Capacity Warning** (6/6 plans) — completed 2026-05-04
  Slots erhalten ein optionales `max_paid_employees: Option<u8>` Capacity-Limit. Backend emittiert nicht-blockierende `Warning::PaidEmployeeLimitExceeded` (a) im `BookingCreateResult.warnings` im Conflict-Aware-Booking-Flow und (b) als `current_paid_count` per Slot im Shiftplan-Week-View. 461 Tests grün; 16/16 D-decisions verified. Frontend (shifty-dioxus) out of scope.

**Full milestone archive:** [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)

</details>

<details>
<summary>✅ v1.2 Frontend rest-types Konsolidierung (Phasen 6–7) — SHIPPED 2026-05-07</summary>

- [x] **Phase 6: rest-types Unification & Frontend Compile-Through** (5/5 plans) — completed 2026-05-07
  Backend-`rest-types` als single source of truth verdrahtet, Frontend-Fork gelöscht, 17 fehlende TOs/Enum-Varianten + 4 fehlende Felder + Match-Arme adressiert; `cargo build --target wasm32-unknown-unknown` grün; 466 Backend-Tests ohne Regression. 8/8 V-Truths verified.
- [x] **Phase 7: Runtime Smoke & Regression Safety** (1/1 plan) — completed 2026-05-07
  Frontend-Boot, Login und Shiftplan-Navigation auf Integrationsumgebung verifiziert (User-UAT 2026-05-07); Backend `cargo check --workspace` + `cargo test --workspace` re-verifiziert (Subsumption von Phase-6 V-Truth #6 + #7 plus lokaler Re-Run). 4/4 Success Criteria verified.

**Full milestone archive:** [`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md)

</details>

## Phase Details

### Phase 8: Absence-CRUD-Page Foundation

**Goal:** Neue Top-Level-Route `absences` in `shifty-backend/shifty-dioxus` ist via Menü erreichbar und bietet vollständiges CRUD gegen `/absence-period`. HR-vs-Employee-Sicht kommt aus dem Auth-Context (kein User-Toggle). Die Form bietet Datum-Range-Picker (Ganztage), Kategorie-Dropdown (`Vacation` / `SickLeave` / `UnpaidLeave`) und Description; nicht-blockierende `AbsencePeriodCreateResultTO.warnings[]` werden gerendert. Zusätzlich wird ein neuer Backend-Resturlaubs-Endpoint nachgezogen, weil `VacationEntitlementCard` und `VacationPerPersonList` aus dem Mockup einen autoritativen Resturlaubs-Wert anzeigen (siehe CONTEXT.md D-02/D-03).

**Depends on:** v1.0 Phase 1 (Absence-Backend-Domain), v1.0 Phase 4 (Cutover-Surface), v1.2 Phase 6 (rest-types-Unification — `AbsencePeriodTO` / `AbsenceCategoryTO` / `AbsencePeriodCreateResultTO` / `WarningTO` aus zentralem `rest-types` referenzierbar), v1.2 Phase 7 (WASM-Compile + Runtime-Smoke grün).

**Requirements:** FUI-A-01, FUI-A-02, FUI-A-03, FUI-A-04

**Success Criteria** (was muss WAHR sein, nachdem die Phase abgeschlossen ist):

1. Route `/absences` ist via Menü erreichbar; HR-Privileg-Check schaltet Filter über alle Mitarbeiter frei (Auth-Context, kein User-Toggle) (FUI-A-01).
2. Form erlaubt CRUD eines `AbsencePeriodTO` mit Datum-Range-Picker (Ganztage), Kategorie-Dropdown (`Vacation` / `SickLeave` / `UnpaidLeave`), Description; Self-Overlap-`422` aus Backend wird als Validation-Error gerendert (FUI-A-02, FUI-A-03).
3. `AbsencePeriodCreateResultTO.warnings[]` aus POST/PUT-Antwort wird als nicht-blockierende Hinweisliste gerendert (FUI-A-04).
4. Neuer Backend-Resturlaubs-Endpoint (Shape Plan-Phase-Decision) liefert für `(sales_person_id [, year])` einen Wert mit entitled / used / planned / remaining (oder semantisch äquivalent); `hr ∨ self`-Permission analog zu `/absence-period`; OpenAPI-`#[utoipa::path]`-Annotation; `ToSchema` auf der DTO. Frontend-Komponenten `VacationEntitlementCard` (eigener User) und `VacationPerPersonList` (HR-Übersicht) konsumieren diesen Endpoint.
5. `cargo build --target wasm32-unknown-unknown` im `shifty-backend/shifty-dioxus/`-Subordner liefert Exit-Code 0 ohne Errors; `cargo check --workspace` + `cargo test --workspace` im Backend-Root grün (Backend-Erweiterung darf keine Regression verursachen); UAT-Smoke gegen Integrationsumgebung (HR + Employee Login je einmal Anlage + Edit + Delete + Resturlaubs-Anzeige) erfolgreich.

**Plans:** 9 plans (3 Backend-Waves + 3 Frontend-Waves + 1 Gap-Closure-Plan + 1 Cutover-Response-Polish-Plan + 1 Cutover-Heuristik-Plan)

- [x] 08-01-PLAN.md — Service-Trait + Domain-Struct (`VacationBalanceService`) + DTO (`VacationBalanceTO`) — Wave 1, BL-Tier interface foundation (completed 2026-05-08)
- [x] 08-02-PLAN.md — Service-Impl (BL-Tier per gen_service_impl!) + 7 Unit-Tests + REST-Endpoints (utoipa) + DI-Wiring in main.rs — Wave 2 (completed 2026-05-08)
- [x] 08-03-PLAN.md — OpenAPI Surface-Assertion-Test (`rest/tests/openapi_surface.rs`; Option-B-Pivot vom flaky insta-snapshot weg; pinnt Pfad-Liste + Schema-Namen + VacationBalance-Tag; 3-run-determinism) — Wave 3 (completed 2026-05-08)
- [x] 08-04-PLAN.md — Frontend Foundation: api.rs (8 fns) + ShiftyError::Validation + state-types + loader + service-coroutines + 60 i18n-Keys (de/en/cs) + Dx-Proxy-Einträge — Wave 4 (completed 2026-05-08)
- [x] 08-05-PLAN.md — AbsencesPage + AbsenceModal + 9 inline components (WarningList + CategoryBadge + StatusPill + VacationEntitlementCard + VacationPerPersonList + AbsenceList + AbsenceFilterBar + StatsGrid + DeleteConfirmDialog) + Route::Absences + TopBar entry + 11 dioxus-ssr snapshot tests + WASM-Build-Gate — Wave 5; closes Wave-0-Item-3 in VALIDATION.md (nyquist_compliant: true) (completed 2026-05-08)
- [x] 08-06-PLAN.md — UAT-Smoke (HR + Employee) + Final-Regression-Gates (cargo test --workspace + WASM-Build) — Wave 6 closure
- [x] 08-07-PLAN.md — Gap-Closure: admin-auto-grant trigger sqlx-Migration + GET /feature-flag/{key} REST-Endpoint + Frontend FeatureFlagsState + TopBar Cutover-Gate + HR-Submenu + Responsive AbsencesPage Layout (completed 2026-05-08)
- [x] 08-08-PLAN.md — Cutover-Response Drift-Details: QuarantineReason::human_text + suggested_action + CutoverQuarantineEntryTO + CutoverRunResultTO.gate_drift_report inline DTO + per-entry quarantined_entries; failed-Gate liefert interpretierbare Antwort ohne File-Lookup (completed 2026-05-08)
- [x] 08-09-PLAN.md — Cutover Wochenpauschalen-Heuristik: detect_weekly_lump_sum + iso_week_range + lookup_active_contract; Migration-Loop bekommt Pre-Check (a.5) VOR Workday/Strict-Match-Quarantäne; 1× extra_hours-Row mit `amount ≈ Σ hours_per_day für Vertragstage der ISO-Woche` → absence_period {Mo, So}; Live-Szenario Max-Schmidt 20h@Friday bei 3-Tage-Vertrag migriert sauber (gate passes, drift=0); 7 unit + 1 integration test (completed 2026-05-08)

**UI hint**: yes

**Notes for plan-phase:** Misch-Phase Backend + Frontend im Monorepo (`shifty-backend/`). **Frontend-Schwerpunkt** (`shifty-dioxus/`): Page + Modal + Service + State + Loader + API-Layer + i18n; Backend-Endpoints `/absence-period` (GET-list, GET-by-id, POST, PUT, DELETE, GET-by-sales-person) sind in v1.0 Phase 1 geshipped (`rest/src/absence.rs`); DTOs (`AbsencePeriodTO`, `AbsenceCategoryTO`, `AbsencePeriodCreateResultTO`, `WarningTO`) liegen in `rest-types/src/lib.rs:1565..2040`. **Backend-Erweiterung neu in Scope:** Resturlaubs-Endpoint + neuer DTO `VacationBalanceTO` (Name + Shape Plan-Phase-Decision) + neuer Service. Erwartete Tier-Klassifizierung: **Business-Logic-Tier** (kombiniert `WorkingHoursService` + `AbsenceService`/`AbsenceDao`, ggf. `SpecialDayService`). Permission `hr ∨ self`. Siehe `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen". Mockup-Quellen: `shifty-backend/shifty-dioxus/shifty-design/project/absences.jsx` (729 Zeilen, `AbsencePage` + `AbsenceModal` + `WarningList` + `CategoryBadge` + `StatusPill` + `VacationEntitlementCard` + `VacationPerPersonList` — alles im Phase-8-Scope) und Integrations-Brief `shifty-backend/shifty-dioxus/shifty-design/project/uploads/absence-feature-frontend.md`. Tweak `viewAs` aus dem Mockup ist NICHT zu übernehmen — Sicht kommt aus Auth-Context (`hr`-Privileg). Confirm-Dialog im Mockup verwendet `window.confirm`; im echten Frontend ist Dioxus-Dialog-Komponente zu nutzen (`shifty-backend/shifty-dioxus/src/component/dialog.rs`). i18n De / En / Cs ist Teil dieser Phase (Page-Titel, Kategorie-Labels, Form-Labels, Warning-Texte) — kein nachgelagerter Audit, sondern direkt mit der Implementierung. Out-of-Scope-Mockup-Komponenten: `UnavailabilityChip` → Phase 10; Deprecation-Banner für legacy `extra_hours` → Phase 11. Vollständige Decision-Liste: `.planning/phases/08-absence-crud-page-foundation/08-CONTEXT.md` (D-01..D-14). Plan-phase legt fest, ob api/loader/state/page-Komponenten und Backend-Erweiterung in einer oder mehreren Waves laufen.

---

### Phase 8.1: Cutover-Migration-UI

**Goal:** Admin-UI im Monorepo (`shifty-backend/shifty-dioxus/`) für die `extra_hours` → `absence_period`-Datenmigration. 3-Stage-Stepper-Wizard (Profile → Dry-Run → Commit) mit Drift-Resolution-Liste und Per-Eintrag-Aktionen (Delete / Edit / Convert-to-Range / Skip) sowie Bulk-Aktionen pro `(sales_person, category, year)`-Gruppe. Schließt den Phase-8-int-UAT-Block, der durch reale Buchungs-Pattern-Diversität entstanden ist (Auto-Heuristik in Plan 08-09 deckt nicht alle Patterns ab — siehe `08-HUMAN-UAT.md` gap-1). Backend-additiv: zwei neue atomic-tx Endpoints (`POST /admin/cutover/convert-quarantine-entry`, `POST /admin/cutover/bulk-convert-quarantine-rows`) auf `CutoverServiceImpl` (Business-Logic-Tier), die `extra_hours`-Soft-Delete + `absence_period`-Insert unter `cutover_admin`-Privileg zusammenführen. Schließungs-Phase für Phase 8 — nach Cutover läuft der dort deferred 35-Schritt-HUMAN-UAT als Subsumption-Plan.

**Depends on:** v1.0 Phase 4 (Cutover-Surface — `CutoverServiceImpl`, `CutoverRunResultTO`, `CUTOVER_ADMIN_PRIVILEGE`), Phase 8 Plan 08-07 (Feature-Flag-Endpoint `GET /feature-flag/{key}` + `FeatureFlagsState` + "Verwaltung"-Submenu-Pattern), Phase 8 Plan 08-08 (`gate_drift_report`-Inline-Shape, `CutoverQuarantineEntryTO`, `QuarantineReason::{human_text, suggested_action}`), Phase 8 Plan 08-09 (`detect_weekly_lump_sum` + `iso_week_range` + `lookup_active_contract`-Helper), v1.2 Phase 6 (rest-types-Cross-Crate-Konstruktion).

**Requirements:** Closure-Phase — kein neues FUI-Requirement; löst Phase-8-Adoption-Block (`08-HUMAN-UAT.md` gap-1). Pflicht-Locale-Coverage (FUI-A-09) gilt für neu hinzugefügte i18n-Keys.

**Success Criteria** (was muss WAHR sein, nachdem die Phase abgeschlossen ist):

1. Admin-Route `/admin/cutover` mit `cutover_admin`-Privileg-Gate (HR sieht Page + Profile + Dry-Run, nur `cutover_admin` darf Commit); 3 sichtbare Stages; Profile + Dry-Run liefern strukturiertes Ergebnis-Display (Quarantine-Counts, Per-Person-Stats, Carryover-Diff).
2. Drift-Resolution-Liste rendert pro `quarantined_entry` ISO-Datum + Wochentag-Code + Hours + Reason-Text + Suggested-Action (alles aus inline `gate_drift_report` von Plan 08-08); gegliedert nach `(sales_person, category, year)`; Per-Eintrag-Aktionen Convert / Edit-extra_hours / Delete / Skip in Action-Spalte.
3. Bulk-Aktion "Alle Wochenpauschalen für (sales_person, category, year) konvertieren" verfügbar je Gruppe; ruft `POST /admin/cutover/bulk-convert-quarantine-rows` (single-Tx, atomar pro Gruppe).
4. Cutover-Commit erst aktiv wenn `quarantined_rows == 0`; Type-to-confirm-Dialog ("CUTOVER") + Migration-Summary vor destruktivem Commit; Idempotenz-Hinweis nach Abschluss + Permanent-Banner bei Re-Open via `absence_range_source_active`-Flag-Check (Plan 08-07 `FeatureFlagsState`).
5. Backend: zwei neue Endpoints (`POST /admin/cutover/convert-quarantine-entry`, `POST /admin/cutover/bulk-convert-quarantine-rows`) mit `#[utoipa::path]`, `ToSchema` auf neuen DTOs, `cutover_admin`-Privilege-Check, atomic-tx (`extra_hours`-Soft-Delete + `absence_period`-Insert in einer Tx); `EXPECTED_PATHS`/`EXPECTED_SCHEMAS` in `rest/tests/openapi_surface.rs` ergänzt; Unit-Tests für beide Service-Methoden.
6. Diagnose-Plan für `08-HUMAN-UAT.md` gap-1 (a) (Vertragsdaten-Edge-Case Lila/Anina/Karin): Reproduce mit Test-Fixtures + Hypothesen (mid-week-Vertragswechsel, Hire-Date-Edge-Cases, Inactive-Contract-Tage in `lookup_active_contract`); Fix wenn klar, sonst dokumentierter bleibender gap.
7. **Optional:** Feiertag-Konsistenz-Fix in `detect_weekly_lump_sum` (gap-1 (c)) ist explizit OUT OF SCOPE (`derive_hours_for_range` skipt Holidays bewusst, `service_impl/src/absence.rs:483-485`); Operator löst manuell via Edit oder Convert + Edit.
8. i18n De / En / Cs vollständig für Page-Chrome (Stage-Labels, Stat-Box-Titel, Action-Buttons, Confirm-Dialog-Texte, Banner-Texte, Toast/Error-Texte); `QuarantineReason`-Texte (`reason_text`, `suggested_action`) bleiben Englisch und werden unverändert gerendert (Plan 08-08-Konvention). Per-Locale-Reference-Matcher-Tests gegen `Locale::En`-statt-`Locale::De`-Bug analog Plan 08-04 D-26.
9. Eigener 8.1-UAT-Plan für die Cutover-UI selbst (Wizard-Stages, Drift-Resolution-Aktionen alle vier je einmal, Bulk-Convert auf Group-Section, Type-to-confirm-Verhalten, Idempotenz-State nach Commit).
10. Phase-8-HUMAN-UAT (35 Schritte, `08-HUMAN-UAT.md`) wird auf int durchlaufen NACH 8.1-UI-Cutover und gemeinsam mit Phase 8.1 closed; gap-1 in `08-HUMAN-UAT.md` auf `resolved` gesetzt.
11. `cargo build --target wasm32-unknown-unknown` im `shifty-backend/shifty-dioxus/`-Subordner liefert Exit-Code 0 ohne Errors; `cargo check --workspace` + `cargo test --workspace` im Backend-Root grün (Backend-Convert-Endpoints + Frontend dürfen keine Regression verursachen).

**Plans:** 12 plans across 6 waves
- [ ] 08.1-01-PLAN.md — rest-types DTOs (4 Request/Response + CutoverConvertErrorTO) — Wave 1
- [ ] 08.1-02-PLAN.md — Service convert_quarantine_entry + compute_gate_diagnostic helper + 4 mockall tests + From-Impl — Wave 1
- [ ] 08.1-03-PLAN.md — Service bulk_convert_quarantine_rows (strict-atomic) + 4 mockall tests + From-Impl — Wave 1
- [ ] 08.1-04-PLAN.md — REST handlers (2) + ApiDoc + OpenAPI surface entries + 5 integration tests — Wave 1
- [ ] 08.1-05-PLAN.md — Frontend api.rs (5 cutover_* fns) + Dioxus.toml proxy entry — Wave 2
- [ ] 08.1-06-PLAN.md — i18n: 33 Cutover* keys × 3 locales + 4 reference-matcher tests — Wave 2
- [ ] 08.1-07-PLAN.md — Router::AdminCutover + page-stub + TopBar Verwaltung-Submenu entry + 4 nav tests — Wave 2
- [ ] 08.1-08-PLAN.md — state/cutover_state + service/cutover (CUTOVER_STORE + CUTOVER_DRIFT_REFRESH + Coroutine) + 5 tests — Wave 2
- [ ] 08.1-09-PLAN.md — page/cutover_admin.rs Single-File-Composition (11 components) + 11 dioxus-ssr snapshot tests + WASM-Build-Gate — Wave 3
- [ ] 08.1-10-PLAN.md — Diagnose-Plan gap-1 (a): Lila/Anina/Karin contract edge-case tests + optional fix — Wave 4
- [ ] 08.1-11-PLAN.md — 8.1-eigener UAT (D-21) — Wave 5 (manual checkpoint)
- [ ] 08.1-12-PLAN.md — Phase-8-HUMAN-UAT-Subsumption (D-20) + final regression gates — Wave 6 (manual checkpoint)

**UI hint**: yes (Frontend-Schwerpunkt + Backend-additiv)

**Notes for plan-phase:** Misch-Phase wie Phase 8 (Backend + Frontend im selben Monorepo). **Vollständige Decision-Liste D-01..D-27 + Phase-Boundary + Out-of-Scope:** `.planning/phases/8.1-cutover-migration-ui/8.1-CONTEXT.md` (CANONICAL — alle Detail-Decisions inkl. Convert-Endpoint-Shapes, Stepper-Topologie D-07/D-08, Drift-Listen-Gliederung D-11..D-14, Type-to-confirm D-15, Idempotenz-Detection D-17, Privilege-Gate D-23, i18n-Pattern D-26/D-27). **Service-Tier:** Convert-Endpoints sind Business-Logic-Tier auf existing `CutoverServiceImpl` (Cross-Aggregat: extra_hours + absence_period + working_hours; reuse `detect_weekly_lump_sum` + `iso_week_range` + `lookup_active_contract`). **Idempotenz-Pattern:** `Option<bool>::None`-Default in `FeatureFlagsState` verhindert Banner-Flackern (Plan 08-07-Pattern). **Auto-Re-Run:** Nach jeder Resolve-Aktion `gate-dry-run` triggern (D-08); Plan-Phase entscheidet ob Backend `refreshed_drift_report` inline mitliefert oder Frontend separat fetched. **OpenAPI-Surface-Test:** `rest/tests/openapi_surface.rs` (`EXPECTED_PATHS` + `EXPECTED_SCHEMAS`) muss um die zwei neuen Pfade + neue DTOs ergänzt werden (Plan 08-03-Pattern). **Snapshot-Schema-Versioning:** 8.1 berührt keine `BillingPeriodValueType`-Erweiterung — `CURRENT_SNAPSHOT_SCHEMA_VERSION` braucht KEINEN Bump. **VCS:** jj-only (siehe `CLAUDE.local.md`); Plans dürfen keine `git commit`-Befehle planen. **Out-of-Scope explizit:** Backend-Heuristik-Fix für Feiertage (D-06), Audit-Log-UI, Cutover-History-Page, Multi-Tenant, Force-Commit-Override (siehe CONTEXT.md `<domain>` "Out of Scope").

---

### Phase 8.2: Manual-Range-Convert für Quarantäne

**Goal:** Closure-Phase für `08.1-10`-gap-1a (Karin-Pattern). Erweitert die in Phase 8.1 etablierte Convert-API um einen manuellen Pfad: Wenn die Heuristik einen Quarantäne-Eintrag nicht auflösen kann (Vertragswechsel mit differing `hours_per_day` mid-week, Hire/End-Date-Edge-Cases ohne weekly-lump-sum-Match), gibt der Admin/HR den Ziel-`absence_period`-Zeitraum direkt vor und das Backend schreibt ohne weitere Mustererkennung. Frontend ersetzt das in 8.1-09 als bekannten Stub belassene `EditExtraHoursModal` durch ein `ManualConvertModal` mit Date-Range-Picker. Audit-Pfad bleibt identisch zum heuristischen Convert (gleicher `synthetic_run_id`-Flow, inline `refreshed_drift_report` per D-08).

**Depends on:** Phase 8.1 (Cutover-Migration-UI — `convert_quarantine_entry`, `compute_gate_diagnostic`, `CutoverAdminPage`-State + Coroutine, `CutoverConvertQuarantineEntryRequest`/`Response`-DTOs).

**Requirements:** Closure-Phase — schließt gap-1a aus `08.1-10-SUMMARY.md` (Karin-Pattern, bleibender gap dokumentiert für Operator-Resolution). Pflicht-Locale-Coverage (FUI-A-09) für neu hinzugefügte i18n-Keys (Date-Range-Picker-Labels, Modal-Titel, Hilfetext).

**Success Criteria:**

1. `CutoverConvertQuarantineEntryRequest` erhält optionales `manual_range: Option<{ start_date, end_date }>`. Bei `Some` skipt `convert_quarantine_entry` die Heuristik (`detect_weekly_lump_sum` + `lookup_active_contract`-Match) und schreibt direkt eine `absence_period` mit dem gegebenen Zeitraum + Soft-Delete der zugehörigen `extra_hours`. Same-Tx, gleicher `synthetic_run_id`, `refreshed_drift_report` inline.
2. Backend-Validation: `start_date <= end_date`; beide Daten innerhalb des Quarantäne-Eintrag-Jahres; Kategorie unverändert (read-only übernommen aus dem Quarantäne-Eintrag); `cutover_admin OR hr` Privilege-Check (D-23 aus 8.1).
3. Karin-Diagnose-Test (`diagnose_int_drift_pattern_karin_*` aus 8.1-10) wird durch einen neuen Test ergänzt: `convert_quarantine_entry` mit `manual_range = Some(...)` löst Karin-Quarantäne ohne Heuristik-Anpassung; `derive_hours_for_range` über die manuell gesetzte Range matcht den `legacy_sum`.
4. Frontend: `EditExtraHoursModal`-Stub aus 8.1-09 wird zum `ManualConvertModal`. Eingabefelder: Datum-von, Datum-bis (`<input type="date">` reicht), Kategorie read-only, Stunden read-only (informativ — die werden vom Backend aus Range + Contract abgeleitet). Submit dispatcht `CutoverAction::ManualConvert { extra_hours_id, manual_range }`. `refreshed_drift_report` aus Response landet im `CUTOVER_STORE` (selbe Mechanik wie 8.1-09 Convert/Bulk-Convert).
5. OpenAPI-Surface-Test bleibt grün — die Änderung ist additiv (neues optionales Feld); ein neuer Schema-Drift-Eintrag für das DTO bestätigt das `manual_range`-Feld.
6. WASM-Build (`cd shifty-dioxus && cargo build --target wasm32-unknown-unknown`) + Backend `cargo test --workspace` grün; Snapshot-Schema-Version unverändert (kein neuer `BillingPeriodValueType`).

**Plans:** 2 plans across 2 waves (sequenziell — Frontend-Plan wartet auf Backend-DTO)
- [x] 08.2-01-PLAN.md — Backend `manual_range`-Branch + DTO + 4 mockall + 1 integration test + OpenAPI-Schema-Update — Wave 1 (completed 2026-05-10, see 08.2-01-SUMMARY.md)
- [x] 08.2-02-PLAN.md — Frontend ManualConvertModal + Coroutine-Action + 8 i18n keys × 3 locales + 4 dioxus-ssr snapshots + WASM-Build-Gate — Wave 2 (completed 2026-05-10, see 08.2-02-SUMMARY.md)

**UI hint**: yes (Backend additiv + Frontend Modal-Erweiterung).

**Notes for plan-phase:** Sehr kleine Phase — voraussichtlich 1-2 Plans (Backend-Erweiterung + Frontend-Modal-Replacement, gegebenenfalls in einer Welle parallelisierbar wenn rest-types-Änderung in Plan 1 gemacht wird). Reuse 8.1-Patterns: Service-Tier-Klassifikation (Business-Logic), `compute_gate_diagnostic` für `refreshed_drift_report`, jj-only Commit-Politik. **Out-of-Scope:** Generelles Edit der Stunden eines `extra_hours`-Eintrags (sofern doch gewünscht: separate Phase oder Kombi-Modal mit Tab-Switch). **Karin-Test wird zur Verifikation des Manual-Convert-Pfads**; bleibender gap aus 8.1 wird als `resolved` markiert in 8.2 SUMMARY.

---

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1 — Absence Domain Foundation | v1.0 | 5/5 | Complete | 2026-05-01 |
| 2 — Reporting Integration & Snapshot Versioning | v1.0 | 4/4 | Complete | 2026-05-02 |
| 3 — Booking & Shift-Plan Konflikt-Integration | v1.0 | 6/6 | Complete | 2026-05-02 |
| 4 — Migration & Cutover | v1.0 | 8/8 | Complete | 2026-05-03 |
| 5 — Slot Paid Capacity Warning | v1.1 | 6/6 | Complete | 2026-05-04 |
| 6 — rest-types Unification & Frontend Compile-Through | v1.2 | 5/5 | Complete | 2026-05-07 |
| 7 — Runtime Smoke & Regression Safety | v1.2 | 1/1 | Complete | 2026-05-07 |
| 8 — Absence-CRUD-Page Foundation | v1.3 | 8/9 | In Progress | — |
| 8.1 — Cutover-Migration-UI | v1.3 | 0/12 | In Progress | — |
| 8.2 — Manual-Range-Convert für Quarantäne | v1.3 | 2/2 | Complete | 2026-05-10 |
| 9 — Booking-Flow Reverse-Warnings + Copy-Week | v1.3 | 0/? | Pending | — |
| 10 — Shiftplan-View Unavailability-Marker | v1.3 | 0/? | Pending | — |
| 11 — Migrations-Hinweis-UX + Deprecation-Handling | v1.3 | 0/? | Pending | — |
| 12 — UI-Closure v1.1/v1.2-Restanten | v1.3 | 0/? | Pending | — |
| 13 — i18n-Vollständigkeits-Audit + v1.3 Smoke-Closure | v1.3 | 0/? | Pending | — |

---

*Last updated: 2026-05-10 — Phase 8.2 verified passed (6/6 must-haves, gsd-verifier 08.2-VERIFICATION.md). Plan 08.2-02 (Frontend ManualConvertModal) complete: shifty-dioxus api::cutover_convert_quarantine_entry um `manual_range: Option<ManualRangeTO>` erweitert (existing ConvertSingle call-site auf `None` migriert); neue `CutoverAction::ConvertSingleManualRange { extra_hours_id, start_date, end_date }`-Variante mit Coroutine-Branch (formatiert dates via time::macros::format_description, baut ManualRangeTO, ruft Backend, P-6 fallback auf separate gate-dry-run wenn refreshed_drift_report.is_none(), schreibt CUTOVER_STORE.last_dry_run + bump_cutover_refresh); ManualConvertModal-Component ersetzt EditExtraHoursModal-Stub (Custom-Backdrop, 2× `<input type="date">`, read-only amount + category als spans D-31/D-32, inline error-rendering, P-7 defense — kein unwrap_or_else hardcoded fallback); DriftEntryRow Edit-Button öffnet ManualConvertModal mit `category: drift_row_meta.0.category` als read-only Quelle (CutoverQuarantineEntryTO hat kein category-Feld); on_submit dispatcht ConvertSingleManualRange + close-on-submit. 8 neue i18n-Keys × 3 Locales (DE/EN/CS — `CutoverManualConvert{ModalTitle,Help,StartLabel,EndLabel,BtnSubmit,ErrStartAfterEnd,ErrYearMismatch,ErrOverlap}`) + Per-Locale-Reference-Matcher-Tests erweitert (Pitfall-2-Guard). 4 neue dioxus-ssr Snapshot-Tests (`manual_convert_modal_renders_two_date_inputs` / `manual_convert_modal_renders_validation_error_when_start_after_end` / `manual_convert_modal_not_rendered_when_closed` / `manual_convert_modal_dispatches_action_on_valid_submit`) ersetzen Test 11 aus 8.1-09 (`edit_extra_hours_modal_renders_amount_and_date_only`); 536/536 shifty-dioxus binary tests grün; cargo check --workspace grün; WASM-Build-Gate (`nix-shell -p openssl pkg-config lld --command "cargo build --target wasm32-unknown-unknown"`) exit 0. 3 jj-commits (`feat`/`feat`/`feat`). Phase 8.2 Plans complete (2/2). Karin-Pattern (gap-1a) jetzt operativ end-to-end auflösbar — UAT-bereit.*
