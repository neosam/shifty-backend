# Phase 8: Absence-CRUD-Page Foundation - Context

**Gathered:** 2026-05-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Neue Top-Level-Route `/absences` im Dioxus-Frontend (`shifty-backend/shifty-dioxus/`) liefert vollständiges CRUD gegen den bestehenden Backend-Endpoint `/absence-period`. HR/Employee-Sicht wird ausschließlich aus dem Auth-Context abgeleitet (kein User-Toggle). Form bietet Datum-Range-Picker (Ganztage), Kategorie-Dropdown (`Vacation`/`SickLeave`/`UnpaidLeave`) und Description; Self-Overlap-`422` wird als Validation-Error gerendert; `AbsencePeriodCreateResultTO.warnings[]` als nicht-blockierende Liste.

**Wichtige Scope-Erweiterung gegenüber ROADMAP-Notes:** Phase 8 ist KEINE reine Frontend-Phase. Sie zieht einen neuen Backend-Endpoint nach (Resturlaubs-Berechnung), weil die Frontend-Komponenten `VacationEntitlementCard` und `VacationPerPersonList` aus dem Mockup einen Backend-berechneten Wert brauchen. Siehe D-03/D-04.

**In Scope:**
- Frontend: `AbsencePage` (Liste mit HR/Employee-Sicht), `AbsenceModal` (Create/Edit/Delete), `WarningList`, `CategoryBadge`, `StatusPill` (Aktiv/Geplant/Beendet), `VacationEntitlementCard` (Resturlaub aktueller User), `VacationPerPersonList` (HR-Übersicht-Kacheln Resturlaub pro Mitarbeiter)
- Backend: neuer Resturlaubs-Endpoint + Service + DTO (Tier-Klassifizierung Plan-Phase-Decision)
- Routing, API-Layer, Loader, State-Module, i18n in De/En/Cs

**Out of Scope (explizit andere Phasen):**
- `UnavailabilityChip` aus dem Mockup → Phase 10 (Shiftplan-View Unavailability-Marker)
- Legacy-`extra_hours`-Deprecation-Banner aus dem Mockup → Phase 11 (Migrations-Hinweis-UX)
- Booking-Flow Reverse-Warnings → Phase 9
- Halbtage/Stundenebene, Genehmigungs-Workflow, Admin-Cutover-UI → grundsätzlich out of scope v1.3

</domain>

<decisions>
## Implementation Decisions

### Mockup-Scope
- **D-01:** Voller Mockup-Umfang für die Absence-Domain wird in Phase 8 implementiert: `AbsencePage` (Liste), `AbsenceModal` (Create/Edit/Delete), `WarningList`, `CategoryBadge`, `StatusPill`, `VacationEntitlementCard`, `VacationPerPersonList`. Der 729-Zeilen-Mockup `absences.jsx` ist visual reference, NICHT 1:1-Portierung — Tweak `viewAs` und `window.confirm` werden bewusst NICHT übernommen.

### Backend-Erweiterung (bricht "reine Frontend-Phase"-Annahme)
- **D-02:** Phase 8 ist Misch-Phase Backend + Frontend. Der `Notes for plan-phase`-Hinweis in ROADMAP.md "Backend bleibt unangetastet" ist obsolet — die Detail-Section sollte vor Plan-Phase nachgezogen werden (siehe `<deferred>`-Sektion: `ROADMAP.md-Update`).
- **D-03:** Backend liefert einen neuen Resturlaubs-Endpoint, weil `VacationEntitlementCard` und `VacationPerPersonList` einen autoritativen Resturlaubs-Wert anzeigen. Plan-Phase entscheidet die genaue Endpoint-Shape (z. B. `/absence-period/vacation-balance/{sales_person_id}` oder `/vacation-balance/{sales_person_id}/{year}`).
- **D-04:** Tier-Klassifizierung des Resturlaubs-Service liegt in der Plan-Phase. Erwartete Klassifizierung: **Business-Logic-Tier**, weil der Service Cross-Entity-Daten kombiniert (`WorkingHours.vacation_days_per_year` ∧ aufsummierte Vacation-`AbsencePeriod`-Tage ∧ ggf. Special-Days/Feiertage). Siehe `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen". Permission ist `hr ∨ self` analog zu AbsencePeriod selbst.

### Frontend-Implementation
- **D-05:** Range-Picker = zwei native `<input type="date">` gekoppelt mit Cross-Field-Validation (`to >= from`). Kein neues `RangePicker`-Atom in Phase 8 — kein neuer reusable Pattern, kein neuer Asset-Footprint.
- **D-06:** HR-Filter-Set: drei Filter — Person-Dropdown + Kategorie-Dropdown + Status (Aktiv/Geplant/Beendet). Status ist im Frontend berechnet aus `from_date`/`to_date` und `today` (clientseitig, kein Backend-Field).
- **D-07:** Edit/Delete = `AbsenceModal` mit Edit-Variante (analog zu `extra_hours_modal`, `contract_modal`, `slot_edit`). Delete-Button im Modal mit Dioxus-Confirm-Dialog (über `component/dialog.rs`, NICHT `window.confirm`).
- **D-08:** `409 Version-Konflikt` (PUT mit veraltetem `$version`) → Banner im Modal "Eintrag wurde anderswo geändert. Erneut laden?" mit Reload-Button. User klärt manuell, Form-State bleibt erhalten bis User entscheidet.

### Sicht-Auswahl & Permission
- **D-09:** HR/Employee-Sicht kommt aus `state/auth_info.rs` `has_privilege("hr")` — kein UI-Toggle, keine User-Preference. Mockup-Tweak `viewAs` ist NICHT zu übernehmen.
- **D-10:** Menü-Eintrag "Abwesenheiten" in `component/top_bar.rs` für ALLE eingeloggten User sichtbar (HR + Employee), HR-Privileg schaltet nur den Filter-Modus innerhalb der Page um (alle Mitarbeiter vs. nur eigene Liste).

### Validation & Warnings
- **D-11:** Self-Overlap-`422` aus Backend wird als Validation-Error im Modal gerendert (Inline-Fehler unter den Range-Feldern, nicht als globaler Toast).
- **D-12:** `AbsencePeriodCreateResultTO.warnings[]` (Forward-Warnings: Booking-Konflikt, Manual-Unavailable-Konflikt) wird nach erfolgreichem POST/PUT im Modal als nicht-blockierende Liste vor dem Modal-Close angezeigt, dann beim User-Acknowledge geschlossen.

### i18n
- **D-13:** i18n-Keys werden inline mit der Implementation hinzugefügt — neue `Key`-Variants in `src/i18n/mod.rs` unter Comment-Block `// Absence management`, sofortige Befüllung in allen drei Locales (`en.rs` / `de.rs` / `cs.rs`). Kein nachgelagerter Audit; Phase 13 (FUI-A-09) ist nur cross-phase Compliance-Gate. Wachsam bleiben gegen den historischen `Locale::En`-statt-`Locale::De`-Bug.

### Dialog & Form-Pattern
- **D-14:** `AbsenceModal` nutzt `component/dialog.rs` (Center-Variante), Form-Atoms aus `component/form/inputs.rs` (TextInput, TextareaInput, SelectInput) + `component/form/field.rs` (Field-Wrapper), Buttons aus `component/atoms/btn.rs` (BtnVariant: Primary für Save, Ghost für Cancel, Danger für Delete).

### Claude's Discretion
- Konkrete CSS/Layout-Details (Spaltenbreiten, Spacing, Tailwind-Utility-Klassen) folgen `input.css` Design-Tokens und `tailwind.config.js`.
- Loader-Side-Joins (z. B. SalesPerson-Cross-Resolve für HR-Liste-Personen-Anzeige) werden in `src/loader.rs` analog bestehenden Patterns implementiert.
- Wave-Topologie und Plan-Aufteilung (api+state in einer Wave, page+component in zweiter Wave, oder alles in einem Plan) entscheidet die Plan-Phase.

### Folded Todos
*Keine Todos in den Phase-8-Scope gefolded — siehe Reviewed Todos in `<deferred>` für die drei keyword-matched, aber inhaltlich nicht passenden Einträge.*

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Roadmap & Requirements
- `.planning/ROADMAP.md` § "Phase Details" → "### Phase 8" — Goal, Success Criteria, Requirements (FUI-A-01..04). HINWEIS: Notes-Block ist obsolet bezüglich "Backend bleibt unangetastet" — siehe D-02.
- `.planning/REQUIREMENTS.md` — FUI-A-01 (Route + Menü), FUI-A-02 (HR/Employee-Sicht), FUI-A-03 (Form), FUI-A-04 (Warnings).
- `.planning/PROJECT.md` § "Current Milestone v1.3" + § "GSD-Scope-Regel" — Phasen umfassen Backend UND Frontend; Plan-Header muss explizit notieren, welche Pfade betroffen sind.
- `.planning/STATE.md` § "Constraints In Force" — VCS=jj, NixOS, i18n-Drei-Locales-Pflicht, rest-types-Cross-Crate-Konstruktion (`default-features = false` im Frontend), Service-Tier-Konvention.

### Backend-Integrations-Brief & Mockup
- `shifty-backend/shifty-dioxus/shifty-design/project/uploads/absence-feature-frontend.md` — vollständiger Backend-Brief mit DTOs, Endpoints, Statuscodes, Permission-Modell, sechs empfohlenen Frontend-Maßnahmen.
- `shifty-backend/shifty-dioxus/shifty-design/project/absences.jsx` (728 Zeilen) — visual reference für `AbsencePage`, `AbsenceModal`, `WarningList`, `CategoryBadge`, `StatusPill`, `VacationEntitlementCard`, `VacationPerPersonList`. NICHT 1:1 portieren (D-09).
- `.planning/notes/abwesenheiten-frontend-context.md` — Briefing-Zusammenfassung, sechs Frontend-Maßnahmen, HR ∨ self-Architekturnotiz.
- `.planning/seeds/abwesenheiten-frontend-milestone.md` — Sub-Phasen-Skizze (Phase A–E); Phase A entspricht Phase 8 + Backend-Erweiterung (D-03/D-04).

### Codebase-Conventions
- `shifty-backend/shifty-dioxus/CLAUDE.md` — Frontend-Konventionen: Component-Service-State-Pattern, i18n in `src/i18n/`, Dioxus 0.6.1 + Tailwind, bekannter `Locale::En`-statt-`Locale::De`-Bug.
- `.planning/codebase/frontend/STRUCTURE.md` — Verzeichnis-Layout, Naming-Conventions, "Where to Add New Code" pro Layer (Page, Component, Service, State, API, i18n, Tests).
- `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen: Basic vs. Business-Logic Services" — VacationBalance ist Cross-Entity-Read → Business-Logic-Tier (D-04).
- `shifty-backend/CLAUDE.md` § "Implementation Patterns" — `gen_service_impl!` Macro, Transaction-Pattern, REST-OpenAPI-Annotation.
- `shifty-backend/CLAUDE.local.md` — jj-only-VCS, NixOS-Toolchain, `nix develop` für `wasm32-unknown-unknown`-Build.

### DTOs & Endpoints
- `rest-types/src/lib.rs:1565..2040` — `AbsencePeriodTO`, `AbsencePeriodCreateResultTO`, `AbsenceCategoryTO`, `WarningTO`. Plan-Phase muss `VacationBalanceTO` (oder ähnlichen Namen) hinzufügen.
- `rest/src/absence.rs` — Backend-Quelle für `/absence-period` Endpoints. Plan-Phase ergänzt hier oder in einer neuen Datei den Resturlaubs-Endpoint.

### Reusable Code-Patterns (siehe code_context)
- `shifty-backend/shifty-dioxus/src/component/dialog.rs` — Modal-Primitive.
- `shifty-backend/shifty-dioxus/src/component/form/inputs.rs`, `field.rs` — Form-Atoms.
- `shifty-backend/shifty-dioxus/src/component/atoms/btn.rs` — Btn + BtnVariant.
- `shifty-backend/shifty-dioxus/src/state/auth_info.rs` — `has_privilege` Pattern.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets (Frontend)
- `src/component/dialog.rs` (Center/Sheet/Bottom/Auto + ESC + scroll lock): wird `AbsenceModal` (Create/Edit/Delete) tragen. Center-Variante.
- `src/component/form/inputs.rs` (TextInput, TextareaInput, SelectInput): für Description-Textarea, Kategorie-SelectInput; Datum-Inputs als native `<input type="date">` (D-05) — falls TextInput erweiterungsfähig auf `type="date"` ist, wiederverwenden, sonst inline.
- `src/component/form/field.rs` (Field-Wrapper für Label + Input): für jedes Form-Feld.
- `src/component/atoms/btn.rs` (BtnVariant Primary/Secondary/Ghost/Danger): Save=Primary, Cancel=Ghost, Delete=Danger.
- `src/component/atoms/person_chip.rs` (PersonChip mit `.person-pill`): für Personen-Anzeige in HR-Liste.
- `src/component/atoms/tuple_row.rs` (Label/Value): ggf. für Detail-View innerhalb des Modals (Created-Zeitstempel etc.).
- `src/state/auth_info.rs` (`has_privilege`): für HR/Employee-Sicht-Auswahl (D-09).

### Established Patterns (Frontend)
- **Page-Service-State-Pattern:** `src/page/absences.rs` (neu) → `src/service/absence.rs` (neu, `AbsenceAction` Enum, `ABSENCE_STORE` GlobalSignal) → `src/state/absence_period.rs` (neu, `AbsencePeriod`-Typ mit `From<&AbsencePeriodTO>` Impl).
- **Routing:** `enum Route` in `src/router.rs` — neuer Variant `Absences`.
- **API-Layer:** `src/api.rs` (1269 Zeilen) — neue async-Funktionen: `list_absence_periods`, `get_absence_period`, `create_absence_period`, `update_absence_period`, `delete_absence_period`, `list_absence_periods_by_sales_person`, plus Resturlaubs-Endpoint (Name in Plan-Phase).
- **Loader:** `src/loader.rs` — TO → state-Mapping mit Side-Joins (z. B. SalesPerson-Cross-Resolve für HR-Liste).
- **Refresh-Token-Pattern:** `ABSENCE_REFRESH: GlobalSignal<u64>` analog `SHIFTPLAN_REFRESH` für Liste-Reload nach POST/PUT/DELETE.
- **i18n-Pattern:** `Key`-Enum-Variants in `src/i18n/mod.rs` unter neuem Comment-Block `// Absence management`, `i18n.add_text(Locale::*, Key::Foo, "…")` in allen drei Locales (`en.rs`, `de.rs`, `cs.rs`).
- **Dx-Proxy:** `Dioxus.toml` — neue `[[web.proxy]]` Einträge für jede neue Backend-Resource.

### Established Patterns (Backend, für Resturlaubs-Endpoint)
- **`gen_service_impl!` Macro** in `service_impl/` — für `VacationBalanceServiceImpl` (Business-Logic-Tier-Klassifizierung erwartet, D-04). Konsumiert `AbsenceDao`/`AbsenceService`, `WorkingHoursService`, `PermissionService`, `TransactionDao`. Genaue Dep-Auswahl ist Plan-Phase.
- **Transaction-Pattern:** `Option<Self::Transaction>` + `transaction_dao.use_transaction(tx).await?` + `commit(tx)`.
- **REST:** Axum-Routing in `rest/src/`, `#[utoipa::path]` Annotation, `ToSchema`-Derive auf `VacationBalanceTO` in `rest-types/src/lib.rs`.
- **DI in `shifty_bin/src/main.rs`:** `VacationBalanceServiceImpl` als Business-Logic-Tier konstruiert (nach Basic-Services und nach `WorkingHoursService` und `AbsenceService`).

### Integration Points
- **Router:** `Route::Absences` → `AbsencesPage` in `src/page/absences.rs`.
- **Top-Bar Menü:** `src/component/top_bar.rs` (1166 Zeilen, Privilege-Gating) — neuer Eintrag "Abwesenheiten" für alle eingeloggten User; HR-Filter-Modus innerhalb der Page (nicht über Menü-Variation).
- **Auth-Gate:** `src/auth.rs` `<Auth>` umschließt Page automatisch via Router; keine Sonderlogik nötig.
- **Workspace-Wiring (Backend):** `Cargo.toml` (Workspace) — Service/Service-Impl-Crate-Eintrag falls neuer Crate (unwahrscheinlich; eher Erweiterung von `service`/`service_impl`). Plan-Phase entscheidet, ob VacationBalance ein eigener File oder Extension von `absence.rs` wird.

</code_context>

<specifics>
## Specific Ideas

- **Visual Reference:** `shifty-backend/shifty-dioxus/shifty-design/project/absences.jsx` ist die visuelle Vorlage. Layout, Farb-Tokens, Component-Struktur orientieren sich daran. Drei explizite Abweichungen: (1) `viewAs`-Tweak ignorieren — Auth-Context entscheidet (D-09); (2) `window.confirm` ersetzen durch Dioxus-Dialog (D-07); (3) Inline-Backend-Mock-Data im Mockup (`window.SHIFTY_DATA`) entfällt — Daten kommen aus `ABSENCE_STORE` und Resturlaubs-Endpoint.
- **Status-Berechnung:** Aktiv/Geplant/Beendet ist client-side berechnet aus `from_date` ≤ `today` ≤ `to_date` (Aktiv), `from_date` > `today` (Geplant), `to_date` < `today` (Beendet). Kein Backend-Field; im Mockup `rangeStatus()`-Funktion (Zeile 26).
- **Resturlaubs-Aggregat-Shape:** Erwartung an `VacationBalanceTO` (Plan-Phase finalisiert): `{ sales_person_id, year, entitled_days, used_days, planned_days, remaining_days }`. Genaue Field-Liste und ob `current as of today` oder `pro Jahr` ist Plan-Phase-Decision (D-04).

</specifics>

<deferred>
## Deferred Ideas

### Mockup-Komponenten in andere Phasen
- **`UnavailabilityChip`** (im Mockup `absences.jsx` Zeile 111): zeigt `UnavailabilityMarkerTO` farbig pro Tag pro Person → Phase 10 (Shiftplan-View Unavailability-Marker, FUI-A-07).
- **Deprecation-Banner** für Legacy-`extra_hours`-Flow (im Mockup): `403 ExtraHoursCategoryDeprecatedErrorTO`-Toast/Banner mit Migrations-Hinweis → Phase 11 (Migrations-Hinweis-UX, FUI-A-08).

### v1.3 Out-of-Scope (festgehalten in REQUIREMENTS.md)
- Halbtage / Stundenebene → Backend modelliert nur Ganztage; wäre Backend-Modell-Änderung.
- Genehmigungs-Workflow → Backend kennt keinen Approval-Schritt.
- Admin-Cutover-UI (`/admin/cutover/*`) → CLI-Flow reicht; eigenes Admin-Surface.

### ROADMAP.md-Update
- Die `### Phase 8`-Detail-Section in `.planning/ROADMAP.md` enthält im `Notes for plan-phase`-Block den Satz "Backend bleibt unangetastet". Dieser Satz ist obsolet wegen D-02/D-03 (Resturlaubs-Endpoint). Die Section sollte vor `/gsd-plan-phase 8` korrigiert werden — entweder durch den User oder durch einen Folge-Edit.

### Reviewed Todos (not folded)
- **`booking-log-service-liefert-sporadisch-500-ohne-logs`** (score 0.9) — Backend-Bug im Booking-Log-Service. Inhaltlich nicht Phase-8-Scope; gehört in einen separaten Bugfix-Phase oder eigenen Plan-Lifecycle.
- **`warnung-eintrag-ausserhalb-vertragszeiten`** (score 0.9) — Booking-Validation gegen Vertragszeiten. Inhaltlich Phase-9-Thema (Booking-Flow Reverse-Warnings) oder eigenes Folge-Feature.
- **`review-frontend-list-user-invitations-silent-empty-fallback`** (score 0.6) — User-Management-Frontend-Bug. Inhaltlich nicht Absence-Domain; eigener Review-Todo-Lifecycle.

</deferred>

---

*Phase: 8-absence-crud-page-foundation*
*Context gathered: 2026-05-07*
