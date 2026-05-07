# Phase 8: Absence-CRUD-Page Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-07
**Phase:** 8-absence-crud-page-foundation
**Areas discussed:** Mockup-Scope, Range-Picker, HR-Filter, Edit/Delete-Flow, 409-Konflikt, Resturlaub-Source

---

## Mockup-Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Kern + Badges (Recommended) | Liste + Modal + CategoryBadge + StatusPill (Aktiv/Geplant/Beendet). VacationEntitlementCard und VacationPerPersonList vertagt. | |
| Nur CRUD MVP | Liste + Modal, ohne Badges/Status-Pills. Schnellster Pfad zur 4er-Success-Criteria-Erfüllung. | |
| Voller Mockup-Umfang | Inkl. VacationEntitlementCard (Resturlaub) und VacationPerPersonList (HR-Übersicht-Kacheln). Mehr Aufwand, mehr i18n. | |
| **Other (free-text)** | "Alles was die Abwesenheitsverwaltung angeht" | ✓ |

**User's choice:** Free-text — "Alles was die Abwesenheitsverwaltung angeht"
**Notes:** Interpretiert als: voller Mockup-Umfang für die Absence-Domain (Liste + Modal + Badges + StatusPill + VacationEntitlementCard + VacationPerPersonList). Aus dem Scope ausgeschlossen: UnavailabilityChip → Phase 10; Deprecation-Banner → Phase 11. Diese Interpretation triggerte die Folgefrage zur Resturlaubs-Source (siehe unten).

---

## Range-Picker

| Option | Description | Selected |
|--------|-------------|----------|
| Zwei <input type=date> gekoppelt (Recommended) | Native Browser-Date-Picker, gekoppelt mit Cross-Field-Validation (to ≥ from). Kein neues Atom nötig, keine externe Dep. | ✓ |
| Neues RangePicker-Atom in component/form/ | Eigene RangePicker-Komponente unter src/component/form/range_picker.rs. Wiederverwendbar, mehr Boilerplate. | |
| Externe Dioxus-Date-Range-Crate | Einbau einer externen Crate. WASM-Compat-Risiko, neue Dependency. | |

**User's choice:** Zwei `<input type="date">` gekoppelt
**Notes:** —

---

## HR-Filter

| Option | Description | Selected |
|--------|-------------|----------|
| Person + Kategorie (Recommended) | Zwei Dropdowns als MVP. Reicht für die HR-Sicht, hält Phase 8 schlank. | |
| Person + Kategorie + Status (Aktiv/Geplant/Beendet) | Mockup-treu inkl. Status-Pill-Filter. Mehr UI-State, aber der Status ist berechenbar aus from/to + heute. | ✓ |
| Nur Person-Dropdown | Minimal. Kategorie kann später nachgezogen werden. Sehr knapp. | |

**User's choice:** Person + Kategorie + Status
**Notes:** Mockup-treu; Status wird client-side aus from_date/to_date + today berechnet.

---

## Edit/Delete-Flow

| Option | Description | Selected |
|--------|-------------|----------|
| Modal (wie Mockup) (Recommended) | AbsenceModal mit Edit-Variante; Delete-Button im Modal mit Dioxus-Confirm-Dialog. Konsistent mit shifty-dioxus-Pattern (extra_hours_modal, contract_modal, slot_edit). | ✓ |
| Inline-Edit in der Tabelle | Editable Row in der Liste, ohne Modal. Kompakt, aber Range-Picker-UX in einer Tabellenzeile ist eng. | |
| Eigene Detail-Route /absences/:id | Wie employee_details: separate Page mit Form. Mehr Routing-Boilerplate, aber bookmarkable URLs. | |

**User's choice:** Modal (wie Mockup)
**Notes:** Window.confirm aus dem Mockup wird durch Dioxus-Dialog ersetzt.

---

## 409-Konflikt

| Option | Description | Selected |
|--------|-------------|----------|
| Banner + Reload-Button (Recommended) | Modal zeigt Banner "Eintrag wurde anderswo geändert. Erneut laden?". User klärt manuell. Sicher, niedrige Komplexität. | ✓ |
| Auto-Reload, Form verworfen | Nach 409: Modal schließt, Liste reloadet automatisch. User-Eingabe geht verloren. Aggressiv. | |
| Vertagt — generischer ShiftyError-Toast | 409 fällt durch zum globalen Error-Handler (ERROR_STORE), generischer Toast. Behandlung in Folge-Phase. Einfachster Pfad. | |

**User's choice:** Banner + Reload-Button
**Notes:** Form-State bleibt erhalten bis User entscheidet.

---

## Resturlaub-Source

| Option | Description | Selected |
|--------|-------------|----------|
| Frontend berechnet (Recommended) | Frontend berechnet aus WorkingHoursTO.vacation_days_per_year minus Summe der Vacation-Tage in AbsencePeriod-Liste. Pragmatisch, kein Backend-Change. | |
| Vertagen — ohne Resturlaubs-Anzeige in Phase 8 | Liste + Modal + Badges + StatusPill ja; VacationEntitlementCard / VacationPerPersonList in eine Folge-Phase. | |
| Backend-Endpoint nachziehen | Phase 8 wird Backend+Frontend; Backend liefert /absence-period/vacation-balance/{sales_person_id}. Bricht den "reine Frontend-Phase"-Scope von v1.3. | ✓ |

**User's choice:** Backend-Endpoint nachziehen
**Notes:** Bricht die "reine Frontend-Phase"-Annahme. ROADMAP-`Notes for plan-phase` ("Backend bleibt unangetastet") ist damit obsolet — siehe CONTEXT.md `<deferred>` § "ROADMAP.md-Update". Service-Tier-Klassifizierung (erwartet Business-Logic-Tier) und genaue Endpoint-Shape (z. B. `/absence-period/vacation-balance/{sales_person_id}` oder `/vacation-balance/{sales_person_id}/{year}`) sind Plan-Phase-Decision.

---

## Claude's Discretion

- CSS/Layout-Details (Spaltenbreiten, Spacing, Tailwind-Utilities) folgen `input.css` Design-Tokens und `tailwind.config.js`.
- Loader-Side-Joins (z. B. SalesPerson-Cross-Resolve für HR-Liste-Personen-Anzeige) analog bestehenden Patterns in `src/loader.rs`.
- Wave-Topologie und Plan-Aufteilung (api+state in einer Wave, page+component in zweiter Wave, oder alles in einem Plan) entscheidet die Plan-Phase.

## Deferred Ideas

- `UnavailabilityChip` (im Mockup) → Phase 10 (Shiftplan-View Unavailability-Marker, FUI-A-07).
- Deprecation-Banner für Legacy-`extra_hours`-Flow (im Mockup) → Phase 11 (Migrations-Hinweis-UX, FUI-A-08).
- Halbtage / Stundenebene → out of scope v1.3 (Backend modelliert nur Ganztage).
- Genehmigungs-Workflow → out of scope v1.3 (Backend kennt keinen Approval-Schritt).
- Admin-Cutover-UI (`/admin/cutover/*`) → out of scope v1.3 (CLI-Flow reicht).
- ROADMAP.md-Update: `### Phase 8`-Detail-Section `Notes for plan-phase` enthält "Backend bleibt unangetastet" — obsolet wegen Backend-Erweiterung; sollte vor Plan-Phase korrigiert werden.

### Reviewed Todos (not folded)

- `booking-log-service-liefert-sporadisch-500-ohne-logs` (Keyword-Match score 0.9) — Backend-Bug, nicht Phase-8-Scope.
- `warnung-eintrag-ausserhalb-vertragszeiten` (Keyword-Match score 0.9) — Booking-Validation, gehört zu Phase 9 oder eigenem Feature.
- `review-frontend-list-user-invitations-silent-empty-fallback` (Keyword-Match score 0.6) — User-Management-Frontend-Bug, nicht Absence-Domain.
