# Requirements: Shifty v1.2

**Defined:** 2026-05-07
**Milestone:** v1.2 Frontend rest-types Konsolidierung
**Goal:** Backend `rest-types` als single source of truth. Frontend kompiliert
wieder gegen die echte API. Keine neuen User-facing Features.

## v1.2 Requirements

### rest-types Unification (RT)

- [ ] **RT-01**: `shifty-dioxus` depends auf die Backend-`rest-types`-Crate
  via `path = "../rest-types"` mit `default-features = false` (kein WASM-
  inkompatibler `service`-Pull-In)
- [ ] **RT-02**: `shifty-dioxus/rest-types/`-Subordner ist gelöscht; nur eine
  `rest-types`-Crate existiert im Repository
- [ ] **RT-03**: Alle bisher fehlenden TOs/Enum-Varianten der Backend-
  `rest-types` (laut `.planning/codebase/frontend/CONCERNS.md` §1: 17
  fehlende Structs/Enums, 4 fehlende Felder, fehlende Match-Arme) sind aus
  dem Frontend-Code referenzierbar

### Frontend Compile (FC)

- [ ] **FC-01**: Match-Arme im Frontend-Code sind erschöpfend für alle vom
  Backend exportierten Enums (`WarningTO`, `ExtraHoursCategoryTO`,
  `InvitationStatus`, etc.) — minimal/no-op-Rendering ist akzeptabel
- [ ] **FC-02**: `cargo build --target wasm32-unknown-unknown` im
  `shifty-dioxus/`-Subordner grün
- [x] **FC-03**: `dx serve` startet das Frontend ohne Runtime-Panics; Login
  + Navigation zur Shiftplan-Seite funktioniert manuell — verifiziert
  durch User-UAT auf Integrationsumgebung 2026-05-07

### Regression Safety (RC)

- [x] **RC-01**: Backend-Workspace `cargo check --workspace` und
  `cargo test --workspace` bleiben grün (keine Regression durch Cross-
  Crate-Änderungen am Backend-`rest-types`) — Phase-6-VERIFICATION V-Truth
  #6 + #7 (466 Tests) plus Re-Verifikation 2026-05-07 zur Phase-7-Closure

## Future Requirements (deferred to v1.3+)

### Frontend User-facing Closure (FUI)

- **FUI-01**: `current_paid_count` als sichtbare Anzeige im Shiftplan-
  Week-View
- **FUI-02**: Capacity-Editor in Slot-Settings für `max_paid_employees`
- **FUI-03**: `VolunteerWork` / `UnpaidLeave` als sichtbare Extra-Hours-
  Kategorien im Frontend-UI
- **FUI-04**: `cap_planned_hours_to_expected` im Frontend-Settings-UI

### Frontend Abwesenheiten-Maske (FUI-A) — v1.3-Kandidat

Frontend-Sicht für die in v1.0 geshippte range-based Absence-Domain.
Backend ist fertig (`/absence-period` CRUD, `WarningTO`,
`UnavailabilityMarkerTO`, Cutover-Flag), Frontend hat aktuell keine
Maske. Quellen: `shifty-dioxus/shifty-design/project/uploads/absence-feature-frontend.md`
(Brief), `shifty-dioxus/shifty-design/project/absences.jsx` (Mockup,
729 Zeilen). Begleitende Note: `notes/abwesenheiten-frontend-context.md`.
Begleitender Seed: `seeds/abwesenheiten-frontend-milestone.md`.

- **FUI-A-01**: Neue Top-Level-Route `absences` (Menü-Eintrag
  "Abwesenheiten") mit CRUD gegen `/absence-period` (POST/GET/GET-by-id/
  PUT/DELETE/GET-by-sales-person)
- **FUI-A-02**: HR-Sicht (Auth-Privileg `hr`) zeigt Liste über alle
  Mitarbeiter mit Filter; Employee-Sicht zeigt nur eigene Einträge.
  Sicht-Auswahl kommt aus dem Auth-Context, nicht aus einem User-Toggle
- **FUI-A-03**: Form-Komponente: Datum-Range-Picker (Ganztage),
  Kategorie-Dropdown (`Vacation` / `SickLeave` / `UnpaidLeave`),
  Description-Feld; Self-Overlap-`422`-Antwort wird als Validation-Error
  gerendert
- **FUI-A-04**: `AbsencePeriodCreateResultTO.warnings[]` aus POST/PUT-
  Antwort wird als nicht-blockierende Hinweisliste angezeigt (Forward-
  Warnings: Booking-Konflikt, Manual-Unavailable-Konflikt)
- **FUI-A-05**: Booking-Flow im Shiftplan-Editor wird auf `POST
  /shiftplan-edit/booking` umgestellt; `BookingCreateResultTO.warnings[]`
  (Reverse-Warnings) werden als nicht-blockierender Confirm-Dialog vor
  finaler Buchung gerendert. `POST /booking` bleibt parallel verfügbar
- **FUI-A-06**: Wochen-Kopie nutzt `POST /shiftplan-edit/copy-week`;
  aggregierte `CopyWeekResultTO.warnings[]` werden zusammengefasst
  angezeigt
- **FUI-A-07**: Shiftplan-Wochen-View nutzt den per-sales-person
  Endpoint und rendert `ShiftplanDayTO.unavailable:
  Option<UnavailabilityMarkerTO>` farbig pro Tag pro Person
  (`absence_period` mit Kategorie-Farbe, `manual_unavailable`,
  `both` mit eigener Visual-Indication)
- **FUI-A-08**: Bestehende "Urlaub eintragen"-Buttons via `extra_hours`
  (in `add_extra_hours_form.rs`, `extra_hours_modal.rs`, etc.) werden
  vor Cutover auf die neue Maske verlinkt; nach Cutover wird
  `403 ExtraHoursCategoryDeprecatedErrorTO` abgefangen und mit User-
  Hinweis auf die neue Maske umgeleitet
- **FUI-A-09**: i18n-Strings (Page-Titel, Kategorie-Labels, Warning-Texte,
  Deprecation-Hinweis) sind in De / En / Cs vollständig (alle drei
  Locales gleichzeitig erweitert, kein Locale::En-statt-Locale::De-Bug
  wie historisch in `de.rs`)

### Backend Slot Constraints (SC)

- **SC-01**: Min-Paid-Capacity Limit pro Slot
- **SC-02**: Skill-Matching pro Slot

## Out of Scope

Explizit ausgeschlossen für v1.2. Begründung jeweils.

| Feature | Reason |
|---------|--------|
| Capacity-Editor UI | User wählte bewusst "rest-types only"-Scope; UI-Closure für v1.3+ |
| Sichtbare `current_paid_count`-Anzeige | Wie oben — kein neues UI in v1.2 |
| OIDC silentRenewIframe Cleanup | Eigener Review-Todo-Lifecycle, kein Bug |
| `list_user_invitations` silent-empty-Fix | Eigener Review-Todo-Lifecycle |
| 04-UAT Test 8 Re-Check | Bei nächster Cutover-Phase neu prüfen |
| `/gsd:secure-phase 04` Nachzug | Bewusstes Skip aus v1.0; Compliance separat klären |

## Traceability

| Requirement | Phase    | Status  |
|-------------|----------|---------|
| RT-01       | Phase 6  | Pending |
| RT-02       | Phase 6  | Pending |
| RT-03       | Phase 6  | Pending |
| FC-01       | Phase 6  | Pending |
| FC-02       | Phase 6  | Pending |
| FC-03       | Phase 7  | Pending |
| RC-01       | Phase 7  | Pending |

**Coverage:**
- v1.2 requirements: 7 total
- Mapped to phases: 7 ✓
- Unmapped: 0

**Phase distribution:**
- Phase 6 (rest-types Unification & Frontend Compile-Through): RT-01, RT-02, RT-03, FC-01, FC-02 (5 requirements — Compile-Gate)
- Phase 7 (Runtime Smoke & Regression Safety): FC-03, RC-01 (2 requirements — Runtime-Gate)

---
*Requirements defined: 2026-05-07*
*Last updated: 2026-05-07 — Traceability filled with Phase-6/Phase-7 mapping (7/7 coverage).*
