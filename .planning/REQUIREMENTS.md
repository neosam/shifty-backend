# Requirements: Shifty v1.3

**Defined:** 2026-05-07
**Milestone:** v1.3 Frontend Abwesenheiten + UI-Closure-Restanten
**Goal:** Dioxus-Frontend liefert sichtbare Abwesenheiten-Maske gegen
`/absence-period` REST-API und schließt die UI-Restanten aus v1.1/v1.2
(sichtbare Capacity-Anzeige, neue Extra-Hours-Kategorien, Settings-Felder).

**Quellen:**

- `notes/abwesenheiten-frontend-context.md` — Briefing
- `seeds/abwesenheiten-frontend-milestone.md` — Sub-Phasen-Skizze (Phase A–E)
- `shifty-dioxus/shifty-design/project/uploads/absence-feature-frontend.md` — Backend-Integrations-Brief
- `shifty-dioxus/shifty-design/project/absences.jsx` — Mockup (729 Zeilen JSX)

REQ-IDs `FUI-A-*` und `FUI-*` setzen die im v1.2-Backlog reservierten IDs
fort (PROJECT.md / MILESTONES.md / STATE.md referenzieren diese Schemata
direkt, deshalb keine Neunummerierung).

## v1.3 Requirements

### Frontend Abwesenheiten-Maske (FUI-A) — Kern

- [ ] **FUI-A-01**: Neue Top-Level-Route `absences` (Menü-Eintrag
  "Abwesenheiten") mit CRUD gegen `/absence-period` (POST/GET-list/
  GET-by-id/PUT/DELETE/GET-by-sales-person)
- [ ] **FUI-A-02**: HR-Sicht (Auth-Privileg `hr`) zeigt Liste über alle
  Mitarbeiter mit Filter; Employee-Sicht zeigt nur eigene Einträge.
  Sicht-Auswahl kommt aus dem Auth-Context, nicht aus einem User-Toggle
- [ ] **FUI-A-03**: Form-Komponente: Datum-Range-Picker (Ganztage),
  Kategorie-Dropdown (`Vacation` / `SickLeave` / `UnpaidLeave`),
  Description-Feld; Self-Overlap-`422`-Antwort wird als Validation-Error
  gerendert
- [ ] **FUI-A-04**: `AbsencePeriodCreateResultTO.warnings[]` aus POST/PUT-
  Antwort wird als nicht-blockierende Hinweisliste angezeigt (Forward-
  Warnings: Booking-Konflikt, Manual-Unavailable-Konflikt)

### Booking-Flow Reverse-Warnings (FUI-A continued)

- [ ] **FUI-A-05**: Booking-Flow im Shiftplan-Editor wird auf
  `POST /shiftplan-edit/booking` umgestellt;
  `BookingCreateResultTO.warnings[]` (Reverse-Warnings) werden als
  nicht-blockierender Confirm-Dialog vor finaler Buchung gerendert
  (Dioxus-Dialog statt `window.confirm`). `POST /booking` bleibt parallel
  für Bestands-Calls
- [~] **FUI-A-06**: ⊘ **DROPPED 2026-06-11** — Wochen-Kopie-UI wurde vom User
  bewusst entfernt (Commit `294566f` "feat: Remove copy last week feature").
  Kein Frontend-Konsument für `POST /shiftplan-edit/copy-week` mehr; der
  Backend-Endpoint + `CopyWeekResultTO` bleiben funktionsfähig. Descope-
  Entscheidung dokumentiert in `phases/09-booking-flow-reverse-warnings-copy-week/09-CONTEXT.md`.
  ~~Wochen-Kopie nutzt `POST /shiftplan-edit/copy-week`; aggregierte
  `CopyWeekResultTO.warnings[]` werden zusammengefasst angezeigt~~

### Shiftplan-View mit Unavailability-Marker

- [ ] **FUI-A-07**: Shiftplan-Wochen-View nutzt den per-sales-person
  Endpoint und rendert `ShiftplanDayTO.unavailable:
  Option<UnavailabilityMarkerTO>` farbig pro Tag pro Person:
  `absence_period` mit Kategorie-Farbe, `manual_unavailable` neutral,
  `both` mit eigener Visual-Indication (signalisiert redundanten
  manuellen Eintrag nach Cutover)

### Migrations-Hinweis-UX

- [ ] **FUI-A-08**: Bestehende "Urlaub eintragen"-Buttons via
  `extra_hours` (in `add_extra_hours_form.rs`, `extra_hours_modal.rs`,
  `add_extra_days_form.rs`, `add_extra_hours_choice.rs`) werden vor
  Cutover auf die neue Maske verlinkt (Soft-Migration); nach Cutover
  wird `403 ExtraHoursCategoryDeprecatedErrorTO` abgefangen und mit
  User-Hinweis auf die neue Maske umgeleitet (Toast/Banner)

### Internationalisierung

- [ ] **FUI-A-09**: i18n-Strings (Page-Titel, Kategorie-Labels,
  Warning-Texte, Deprecation-Hinweise, Form-Labels, Dialog-Texte) sind
  in De / En / Cs vollständig — alle drei Locales gleichzeitig
  erweitert, kein Locale::En-statt-Locale::De-Bug wie historisch in
  `de.rs`

### Halbtag-Abwesenheiten

- [ ] **FUI-A-10**: `AbsencePeriod` unterstützt halbe Urlaubstage über ein
  `day_fraction: Full | Half`-Feld auf der Buchung (nicht auf dem Tag).
  Backend-Datenmodell (DB-Spalte + DTO + Service + REST) wird erweitert;
  Reporting (`derive_hours_for_range` + Vacation-Aggregation in
  `BillingPeriod`) berücksichtigt `Half` als 0,5 Tag bzw. halbe Soll-Stunden;
  `CURRENT_SNAPSHOT_SCHEMA_VERSION` wird gebumpt. Frontend-CRUD (Absence-
  Modal in Phase 8) und Cutover-Migration-UI (Phase 8.1) bekommen Halb/Ganz-
  Eingabe pro Eintrag. Auflösung Vormittag/Nachmittag ist explizit
  ausgeschlossen — `Half` ohne Tageszeit-Disambiguierung. Stundenebene
  bleibt out-of-scope. Konkrete Anwendung: Heiligabend + Silvester. Revidiert
  die alte Out-of-Scope-Entscheidung "Halbtage / Stundenebene für
  Abwesenheiten" — siehe `.planning/notes/halftime-absence-decision.md`.

### UI-Closure aus v1.1/v1.2 (FUI)

- [ ] **FUI-01**: `current_paid_count` als sichtbare Anzeige im
  Shiftplan-Week-View pro Slot; Layout-Variante mit `max_paid_employees`
  (z. B. `2/3 bezahlt`) wenn Limit konfiguriert
- [ ] **FUI-02**: Capacity-Editor in Slot-Settings für
  `max_paid_employees: Option<u8>` (NULL = kein Limit; Editor erlaubt
  Clear-Button für `None`)
- [ ] **FUI-03**: `VolunteerWork` / `UnpaidLeave` als sichtbare
  Extra-Hours-Kategorien im Frontend-UI (heute no-op-Match-Arme via
  `rsx! { "" }` aus v1.2 Plan 06-04)
- [ ] **FUI-04**: `cap_planned_hours_to_expected` als Toggle im
  Frontend-Settings-UI für Sales-Persons sichtbar/editierbar machen

## Out of Scope

Explizit ausgeschlossen für v1.3. Begründung jeweils.

| Feature | Reason |
|---------|--------|
| Stundenebene für Abwesenheiten (z. B. 3 h Arzttermin als Vacation) | Maximalflexibles Modell wäre kompletter Service-Schicht-Umbau + Reporting-Arithmetik in Stunden; konkreter Bedarf bislang nicht aufgetreten. Halbtag-Teilrevision: siehe FUI-A-10 / Phase 8.3, Halbtage werden über `day_fraction: Full \| Half` umgesetzt (Decision Log: `.planning/notes/halftime-absence-decision.md`) |
| Genehmigungs-Workflow für Abwesenheiten | Backend kennt keinen Approval-Schritt; Anlage ist atomar mit `hr ∨ self`-Permission |
| Admin-Cutover-UI (`/admin/cutover/*`) | Getrenntes Admin-Surface; CLI-Flow reicht weiter (Phase E im Seed bewusst optional) |
| Min-Paid-Capacity / Skill-Matching (SC-01, SC-02) | Backend-Themen für künftiges Backend-Milestone, nicht Frontend |
| OIDC silentRenewIframe Cleanup | Eigener Review-Todo-Lifecycle, kein Bug |
| `list_user_invitations` silent-empty-Fix | Eigener Review-Todo-Lifecycle |
| 04-UAT Test 8 Re-Check | Bei nächster Cutover-Phase neu prüfen |
| `/gsd:secure-phase 04` Nachzug | Bewusstes Skip aus v1.0; Compliance separat klären |

## Future Requirements (deferred to v1.4+)

### Backend Slot Constraints (SC)

- **SC-01**: Min-Paid-Capacity Limit pro Slot
- **SC-02**: Skill-Matching pro Slot

## Traceability

| Requirement | Phase    | Status  |
|-------------|----------|---------|
| FUI-A-01    | Phase 8  | Pending |
| FUI-A-02    | Phase 8  | Pending |
| FUI-A-03    | Phase 8  | Pending |
| FUI-A-04    | Phase 8  | Pending |
| FUI-A-05    | Phase 9  | Pending |
| FUI-A-06    | Phase 9  | ⊘ Dropped |
| FUI-A-07    | Phase 10 | Pending |
| FUI-A-08    | Phase 11 | Pending |
| FUI-01      | Phase 12 | Pending |
| FUI-02      | Phase 12 | Pending |
| FUI-03      | Phase 12 | Pending |
| FUI-04      | Phase 12 | Pending |
| FUI-A-09    | Phase 13 | Pending |
| FUI-A-10    | Phase 8.3 | Pending |

**Coverage:**
- v1.3 requirements: 14 total (13 aktiv, FUI-A-06 ⊘ dropped 2026-06-11)
- Mapped to phases: 14 ✓
- Unmapped: 0

**Phase distribution:**
- Phase 8 (Absence-CRUD-Page Foundation): FUI-A-01..04 (4 reqs — Kern)
- Phase 8.3 (Halbtag-Support für Absences): FUI-A-10 (1 req — scope-revision aus Out-of-Scope)
- Phase 9 (Booking-Flow Reverse-Warnings): FUI-A-05 (1 req aktiv; FUI-A-06 ⊘ dropped — Copy-Week descoped)
- Phase 10 (Shiftplan-View Unavailability-Marker): FUI-A-07 (1 req)
- Phase 11 (Migrations-Hinweis-UX + Deprecation-Handling): FUI-A-08 (1 req)
- Phase 12 (UI-Closure v1.1/v1.2-Restanten): FUI-01..04 (4 reqs)
- Phase 13 (i18n-Vollständigkeits-Audit + v1.3 Smoke-Closure): FUI-A-09 (1 req)

---
*Requirements defined: 2026-05-07*
*Last updated: 2026-05-17 — FUI-A-10 (Halbtag-Abwesenheiten) ergänzt, Phase-8.3-Mapping (14/14 coverage)*
