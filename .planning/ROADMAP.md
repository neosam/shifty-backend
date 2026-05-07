# Roadmap: Shifty Backend

## Milestones

- ✅ **v1.0 Range-Based Absence Management** — Phasen 1–4 (shipped 2026-05-03) — siehe [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Slot Capacity & Constraints** — Phase 5 (shipped 2026-05-04) — siehe [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)
- ✅ **v1.2 Frontend rest-types Konsolidierung** — Phasen 6–7 (shipped 2026-05-07) — siehe [`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md)
- ◆ **v1.3 Frontend Abwesenheiten + UI-Closure-Restanten** — Phasen 8–13 (active, started 2026-05-07)

## Phases

### v1.3 Frontend Abwesenheiten + UI-Closure-Restanten (active)

- [ ] **Phase 8: Absence-CRUD-Page Foundation** (Frontend)
  Neue Top-Level-Route `absences` mit CRUD gegen `/absence-period`, HR/Employee-Sicht aus Auth-Context, Form (Range-Picker + Kategorie + Description) und Forward-Warnings-Anzeige.
  Requirements: FUI-A-01, FUI-A-02, FUI-A-03, FUI-A-04
  Success Criteria:
  1. Route `/absences` ist via Menü erreichbar; HR-Privileg-Check schaltet Filter über alle Mitarbeiter frei (Auth-Context, kein User-Toggle)
  2. Form erlaubt CRUD eines `AbsencePeriodTO` mit Datum-Range-Picker (Ganztage), Kategorie-Dropdown (`Vacation`/`SickLeave`/`UnpaidLeave`), Description; Self-Overlap-`422` wird als Validation-Error gerendert
  3. `AbsencePeriodCreateResultTO.warnings[]` aus POST/PUT-Antwort wird als nicht-blockierende Hinweisliste angezeigt
  4. `cargo build --target wasm32-unknown-unknown` grün; UAT-Smoke gegen Integrationsumgebung (HR + Employee Login je einmal Anlage + Edit + Delete)

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

**Plans:** TBD

**UI hint**: yes

**Notes for plan-phase:** Misch-Phase Backend + Frontend im Monorepo (`shifty-backend/`). **Frontend-Schwerpunkt** (`shifty-dioxus/`): Page + Modal + Service + State + Loader + API-Layer + i18n; Backend-Endpoints `/absence-period` (GET-list, GET-by-id, POST, PUT, DELETE, GET-by-sales-person) sind in v1.0 Phase 1 geshipped (`rest/src/absence.rs`); DTOs (`AbsencePeriodTO`, `AbsenceCategoryTO`, `AbsencePeriodCreateResultTO`, `WarningTO`) liegen in `rest-types/src/lib.rs:1565..2040`. **Backend-Erweiterung neu in Scope:** Resturlaubs-Endpoint + neuer DTO `VacationBalanceTO` (Name + Shape Plan-Phase-Decision) + neuer Service. Erwartete Tier-Klassifizierung: **Business-Logic-Tier** (kombiniert `WorkingHoursService` + `AbsenceService`/`AbsenceDao`, ggf. `SpecialDayService`). Permission `hr ∨ self`. Siehe `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen". Mockup-Quellen: `shifty-backend/shifty-dioxus/shifty-design/project/absences.jsx` (729 Zeilen, `AbsencePage` + `AbsenceModal` + `WarningList` + `CategoryBadge` + `StatusPill` + `VacationEntitlementCard` + `VacationPerPersonList` — alles im Phase-8-Scope) und Integrations-Brief `shifty-backend/shifty-dioxus/shifty-design/project/uploads/absence-feature-frontend.md`. Tweak `viewAs` aus dem Mockup ist NICHT zu übernehmen — Sicht kommt aus Auth-Context (`hr`-Privileg). Confirm-Dialog im Mockup verwendet `window.confirm`; im echten Frontend ist Dioxus-Dialog-Komponente zu nutzen (`shifty-backend/shifty-dioxus/src/component/dialog.rs`). i18n De / En / Cs ist Teil dieser Phase (Page-Titel, Kategorie-Labels, Form-Labels, Warning-Texte) — kein nachgelagerter Audit, sondern direkt mit der Implementierung. Out-of-Scope-Mockup-Komponenten: `UnavailabilityChip` → Phase 10; Deprecation-Banner für legacy `extra_hours` → Phase 11. Vollständige Decision-Liste: `.planning/phases/08-absence-crud-page-foundation/08-CONTEXT.md` (D-01..D-14). Plan-phase legt fest, ob api/loader/state/page-Komponenten und Backend-Erweiterung in einer oder mehreren Waves laufen.

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
| 8 — Absence-CRUD-Page Foundation | v1.3 | 0/? | Pending | — |
| 9 — Booking-Flow Reverse-Warnings + Copy-Week | v1.3 | 0/? | Pending | — |
| 10 — Shiftplan-View Unavailability-Marker | v1.3 | 0/? | Pending | — |
| 11 — Migrations-Hinweis-UX + Deprecation-Handling | v1.3 | 0/? | Pending | — |
| 12 — UI-Closure v1.1/v1.2-Restanten | v1.3 | 0/? | Pending | — |
| 13 — i18n-Vollständigkeits-Audit + v1.3 Smoke-Closure | v1.3 | 0/? | Pending | — |

---

*Last updated: 2026-05-07 — v1.3 gestartet via `/gsd-new-milestone v1.3`. Phasen 8–13 abgeleitet aus 13 Requirements (FUI-A-01..09, FUI-01..04). 13/13 Coverage ✓.*
