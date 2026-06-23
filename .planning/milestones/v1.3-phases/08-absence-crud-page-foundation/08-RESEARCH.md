# Phase 8: Absence-CRUD-Page Foundation - Research

**Researched:** 2026-05-08
**Domain:** Misch-Phase Backend + Frontend (Dioxus 0.6.1 / WASM + Axum / Service-Tier)
**Confidence:** HIGH

---

## Summary

Phase 8 ist eine Misch-Phase. Im Frontend (`shifty-backend/shifty-dioxus/`) wird eine neue Top-Level-Route `/absences` mit Page (`AbsencePage`), Modal (`AbsenceModal`), Liste, Filter-Bar, `WarningList`, `CategoryBadge`, `StatusPill`, `VacationEntitlementCard` und `VacationPerPersonList` ausgebaut — alle Komponenten lokal in der Page-Datei, keine neuen wiederverwendbaren Atoms. Im Backend wird ein neuer Resturlaubs-Endpoint nachgezogen, weil die zwei Vacation-Komponenten autoritative Werte brauchen — Empfehlung **Business-Logic-Tier-Service** mit DTO-Wrapper, der `EmployeeReportTO` (oder eine neue `VacationBalanceTO`-Aggregation) liefert.

Die Code-Basis ist reif: alle benötigten DTOs liegen in `rest-types/src/lib.rs:1565..2040` und sind bereits gegen Backend-Service-Trait gegated (`feature = "service-impl"`). Der Frontend-Build konsumiert `rest-types` ohne `service-impl`-Feature (`default-features = false`) — etabliertes Pattern aus v1.2. `TextInput` unterstützt `input_type = "date"` (verifiziert in `inputs.rs:14-30`); ein neues `RangePicker`-Atom ist nicht nötig (D-05). Der Page-Service-State-Pattern ist 1:1 aus `service/employee.rs` + `page/employee_details.rs` übernehmbar (Coroutine + GlobalSignal-Store + Action-Enum + `bump_*_refresh`-Helper).

**Primary recommendation:** Wave-Topologie mit zwei Waves — Wave 1: Backend-Resturlaubs-Endpoint + DTO-Erweiterung in `rest-types/src/lib.rs`; Wave 2: Frontend Page + Modal + Service + State + Loader + API-Layer + i18n + Routing/Top-Bar — Wave 2 darf erst nach Wave 1 starten, weil VacationEntitlementCard ohne Backend-Endpoint keine sinnvolle UAT erlaubt. Innerhalb Wave 2 sind api/state/service als ein Plan und page/components/i18n/router als zweiter Plan realistisch.

---

## User Constraints (from CONTEXT.md)

### Locked Decisions

**Mockup-Scope**

- **D-01:** Voller Mockup-Umfang für die Absence-Domain wird in Phase 8 implementiert: `AbsencePage` (Liste), `AbsenceModal` (Create/Edit/Delete), `WarningList`, `CategoryBadge`, `StatusPill`, `VacationEntitlementCard`, `VacationPerPersonList`. Der 729-Zeilen-Mockup `absences.jsx` ist visual reference, NICHT 1:1-Portierung — Tweak `viewAs` und `window.confirm` werden bewusst NICHT übernommen.

**Backend-Erweiterung (bricht "reine Frontend-Phase"-Annahme)**

- **D-02:** Phase 8 ist Misch-Phase Backend + Frontend. Der `Notes for plan-phase`-Hinweis in ROADMAP.md "Backend bleibt unangetastet" ist obsolet — die Detail-Section sollte vor Plan-Phase nachgezogen werden (siehe `<deferred>`-Sektion: `ROADMAP.md-Update`).
- **D-03:** Backend liefert einen neuen Resturlaubs-Endpoint, weil `VacationEntitlementCard` und `VacationPerPersonList` einen autoritativen Resturlaubs-Wert anzeigen. Plan-Phase entscheidet die genaue Endpoint-Shape (z. B. `/absence-period/vacation-balance/{sales_person_id}` oder `/vacation-balance/{sales_person_id}/{year}`).
- **D-04:** Tier-Klassifizierung des Resturlaubs-Service liegt in der Plan-Phase. Erwartete Klassifizierung: **Business-Logic-Tier**, weil der Service Cross-Entity-Daten kombiniert (`WorkingHours.vacation_days_per_year` ∧ aufsummierte Vacation-`AbsencePeriod`-Tage ∧ ggf. Special-Days/Feiertage). Siehe `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen". Permission ist `hr ∨ self` analog zu AbsencePeriod selbst.

**Frontend-Implementation**

- **D-05:** Range-Picker = zwei native `<input type="date">` gekoppelt mit Cross-Field-Validation (`to >= from`). Kein neues `RangePicker`-Atom in Phase 8.
- **D-06:** HR-Filter-Set: drei Filter — Person-Dropdown + Kategorie-Dropdown + Status (Aktiv/Geplant/Beendet). Status ist im Frontend berechnet aus `from_date`/`to_date` und `today` (clientseitig, kein Backend-Field).
- **D-07:** Edit/Delete = `AbsenceModal` mit Edit-Variante (analog zu `extra_hours_modal`, `contract_modal`, `slot_edit`). Delete-Button im Modal mit Dioxus-Confirm-Dialog (über `component/dialog.rs`, NICHT `window.confirm`).
- **D-08:** `409 Version-Konflikt` (PUT mit veraltetem `$version`) → Banner im Modal "Eintrag wurde anderswo geändert. Erneut laden?" mit Reload-Button. User klärt manuell, Form-State bleibt erhalten bis User entscheidet.

**Sicht-Auswahl & Permission**

- **D-09:** HR/Employee-Sicht kommt aus `state/auth_info.rs` `has_privilege("hr")` — kein UI-Toggle, keine User-Preference.
- **D-10:** Menü-Eintrag "Abwesenheiten" in `component/top_bar.rs` für ALLE eingeloggten User sichtbar (HR + Employee), HR-Privileg schaltet nur den Filter-Modus innerhalb der Page um.

**Validation & Warnings**

- **D-11:** Self-Overlap-`422` aus Backend wird als Validation-Error im Modal gerendert (Inline-Fehler unter den Range-Feldern).
- **D-12:** `AbsencePeriodCreateResultTO.warnings[]` (Forward-Warnings) wird nach erfolgreichem POST/PUT im Modal als nicht-blockierende Liste vor dem Modal-Close angezeigt.

**i18n & Form**

- **D-13:** i18n-Keys werden inline mit der Implementation hinzugefügt (alle drei Locales `en.rs`/`de.rs`/`cs.rs` gleichzeitig befüllt). Wachsam gegen den historischen `Locale::En`-statt-`Locale::De`-Bug.
- **D-14:** `AbsenceModal` nutzt `component/dialog.rs` (Center-Variante), Form-Atoms aus `component/form/inputs.rs` + `component/form/field.rs`, Buttons aus `component/atoms/btn.rs`.

### Claude's Discretion

- Konkrete CSS/Layout-Details (Spaltenbreiten, Spacing, Tailwind-Utility-Klassen) folgen `input.css` Design-Tokens und `tailwind.config.js`.
- Loader-Side-Joins (z. B. SalesPerson-Cross-Resolve für HR-Liste-Personen-Anzeige) werden in `src/loader.rs` analog bestehenden Patterns implementiert.
- Wave-Topologie und Plan-Aufteilung entscheidet die Plan-Phase.

### Deferred Ideas (OUT OF SCOPE)

- **`UnavailabilityChip`** (im Mockup `absences.jsx` Zeile 111) → Phase 10 (FUI-A-07).
- **Deprecation-Banner** für Legacy-`extra_hours`-Flow → Phase 11 (FUI-A-08).
- **v1.3 Out-of-Scope:** Halbtage / Stundenebene; Genehmigungs-Workflow; Admin-Cutover-UI.
- **Reviewed Todos (not folded):** booking-log-500-ohne-logs (Bugfix-Phase), warnung-eintrag-ausserhalb-vertragszeiten (Phase 9), review-frontend-list-user-invitations-silent-empty-fallback (eigener Lifecycle).
- **ROADMAP.md-Update:** "Backend bleibt unangetastet"-Satz im `Notes for plan-phase`-Block ist obsolet — sollte korrigiert werden.

---

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FUI-A-01 | Neue Top-Level-Route `absences` (Menü-Eintrag "Abwesenheiten") mit CRUD gegen `/absence-period` (POST/GET-list/GET-by-id/PUT/DELETE/GET-by-sales-person) | Backend-Endpoints existieren ab v1.0 Phase 1 (`rest/src/absence.rs:30-41`); Frontend-Routing-Pattern in `src/router.rs:19-51` (12 Routes); Top-Bar-Privilege-Gating in `top_bar.rs:32-45`; D-10 macht Menü für alle Eingeloggten sichtbar (kein Privilege-Gate für Eintrag) |
| FUI-A-02 | HR-Sicht (Auth-Privileg `hr`) zeigt Liste über alle Mitarbeiter mit Filter; Employee-Sicht zeigt nur eigene Einträge. Sicht-Auswahl aus Auth-Context | `AuthInfo::has_privilege("hr")` in `state/auth_info.rs:24-26`, AUTH-GlobalSignal in `service/auth.rs:50`. HR konsumiert `GET /absence-period`; Employee konsumiert `GET /absence-period/by-sales-person/{sales_person_id}` |
| FUI-A-03 | Form-Komponente: Datum-Range-Picker (Ganztage), Kategorie-Dropdown (Vacation/SickLeave/UnpaidLeave), Description-Feld; Self-Overlap-`422` als Validation-Error | `TextInput` mit `input_type="date"` verifiziert in `inputs.rs:14-30`; `SelectInput` für Kategorie; `Field` mit `error`-Slot (`field.rs:21-22`); Backend liefert `422 ServiceError::ValidationError` mit `OverlappingPeriod(uuid)` (`rest/src/lib.rs:176-181`, `service_impl/src/absence.rs:204-205`) |
| FUI-A-04 | `AbsencePeriodCreateResultTO.warnings[]` aus POST/PUT-Antwort als nicht-blockierende Hinweisliste | Backend liefert Wrapper-DTO `AbsencePeriodCreateResultTO { absence, warnings }` (`rest-types/src/lib.rs:1862-1866`); `WarningTO` enum mit `AbsenceOverlapsBooking` und `AbsenceOverlapsManualUnavailable` Varianten (`rest-types/src/lib.rs:1693-1706`) |

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Absence CRUD REST endpoints | Backend (Axum REST) | — | Bereits in v1.0 Phase 1 implementiert; Phase 8 konsumiert nur |
| Absence Form-State (mode, fields, validation) | Browser (Dioxus Page-lokal) | — | `use_signal` lokal in `AbsencePage`; verworfen bei Modal-Close (UI-SPEC State-Pattern) |
| Cross-Field-Validation `to >= from` | Browser | — | Client-side bei `oninput`; kein Backend-Round-Trip nötig |
| Status-Berechnung Aktiv/Geplant/Beendet | Browser | — | Client-side aus `from_date`/`to_date` + `today` (D-06, UI-SPEC) — kein Backend-Field |
| Self-Overlap-Detection | Backend (Service-Layer) | Frontend (Anzeige) | `service_impl::absence::create()` führt `find_overlapping`-Check durch (D-Phase1-15); Frontend rendert nur 422 |
| Forward-Warnings (Booking-Konflikt etc.) | Backend (`AbsenceService`) | Frontend (Anzeige) | Forward-Warning-Loop in `create()`/`update()` etabliert seit v1.0 Phase 3 |
| Resturlaubs-Berechnung (entitled / used / planned / remaining) | Backend (Business-Logic-Service) | Frontend (Display) | Cross-Entity-Read: `WorkingHoursService.vacation_days_for_year(year)` + `AbsenceService.find_by_sales_person()` + `CarryoverService.get_carryover()` |
| HR/Employee-Sicht-Switch | Browser (`AUTH.read().auth_info.has_privilege("hr")`) | — | Auth-Context wird beim Login geladen (`service/auth.rs:9-42`); kein Backend-Round-Trip pro Render |
| List-Filter (Person/Kategorie/Status) | Browser | — | Clientseitig auf bereits-geladener Liste (D-06 macht Status explizit clientseitig) |
| i18n-Resolution | Browser | — | `I18N`-GlobalSignal mit `Key`-Enum + Locale-Tabellen — bereits etabliert |
| Optimistic Locking (`$version`) | Backend (DAO + Service) | Frontend (Banner-Anzeige) | Backend liefert `409 EntityConflicts(...)` (`rest/src/lib.rs:170-175`); Frontend zeigt Reload-Banner (D-08) |

---

## Standard Stack

### Core (Frontend — bereits installiert)

| Library | Version | Purpose | Why Standard | Source |
|---------|---------|---------|--------------|--------|
| dioxus | 0.6.1 | UI framework (RSX, Router, Signals, Coroutines) | Etabliert seit Phase 1 | `Cargo.toml:10` `[VERIFIED: codebase grep]` |
| dioxus-ssr | 0.6 | SSR für Dev-Tests von Components | Etabliert; `dialog.rs:461`, `inputs.rs:147` | `Cargo.toml:81` `[VERIFIED]` |
| reqwest | 0.12.15 | HTTP-Client (mit JSON-Support) | Etabliert; alle `api.rs`-Funktionen nutzen es | `Cargo.toml:15` `[VERIFIED]` |
| serde / serde_json | 1.0 / 1.0 | (De-)Serialization | Etabliert | `Cargo.toml:16-17` `[VERIFIED]` |
| time | 0.3.41 | `time::Date` mit serde + parsing | Etabliert; `AbsencePeriodTO.from_date: time::Date` (`rest-types/src/lib.rs:1602`) | `Cargo.toml:35-44` `[VERIFIED]` |
| uuid | 1.17 (features `v4` + `js`) | UUID-Generierung im WASM | Etabliert; `js`-Feature ist WASM-Pflicht | `Cargo.toml:32-34` `[VERIFIED]` |
| futures-util | 0.3.30 | `StreamExt` für `UnboundedReceiver` in Coroutine-Services | Etabliert in `service/employee.rs:1` | `Cargo.toml:21` `[VERIFIED]` |
| tracing | 0.1.41 | Logging | Etabliert (`info!` in `api.rs`, services) | `Cargo.toml:13` `[VERIFIED]` |
| rest-types | path-dep | DTO-Definitionen (single source of truth) | Etabliert seit v1.2 Phase 6, **`default-features = false`** Pflicht | `Cargo.toml:28-30` `[VERIFIED — STATE.md Constraint]` |

### Core (Backend — bereits installiert)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| axum | (workspace) | REST framework | Etabliert |
| utoipa | (workspace) | OpenAPI annotations (`#[utoipa::path]`, `ToSchema`) | Etabliert; CLAUDE.md Pflicht |
| sqlx | (workspace) | DB-Layer mit compile-time-checking | Etabliert |
| async-trait | (workspace) | Trait-async-fn | Etabliert; alle Service-Traits |
| mockall | (workspace) | `#[automock]` für Service-Mocking in Tests | Etabliert (`absence.rs:136`) |

### Supporting (Frontend, bereits in Tree)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `Dialog` | `component/dialog.rs` (688 LOC, 30 Tests) | Modal mit Center-Variante, ESC, Backdrop, Focus | `AbsenceModal` + Delete-Confirm-Dialog |
| `TextInput` | `component/form/inputs.rs:14-53` | `<input>` mit `input_type`-Prop (text/date/datetime-local/number) | Range-Felder Von/Bis (`input_type="date"`) |
| `SelectInput` | `component/form/inputs.rs:55-94` | `<select>` mit Tailwind-Custom-Chevron | Kategorie-Dropdown, Person-Filter, Status-Filter |
| `TextareaInput` | `component/form/inputs.rs:96-138` | `<textarea>` mit min-h und resize-vertical | Description-Feld |
| `Field` | `component/form/field.rs` | Label-Wrapper mit `hint`/`error`-Slot, `span: Option<u8>` für Grid-Span | Jedes Form-Feld; Cross-Field-Error im Bis-Feld |
| `Btn` + `BtnVariant` | `component/atoms/btn.rs` | Primary/Secondary/Ghost/Danger | Save=Primary, Cancel=Ghost, Delete=Danger |
| `PersonChip` | `component/atoms/person_chip.rs` | Person-Pill (Avatar + Name) | HR-Liste Personen-Anzeige (Plan-Phase entscheidet ob direkt) |
| AUTH-State | `service/auth.rs` `AUTH: GlobalSignal<AuthStore>` | Auth-Info-Read | Sicht-Switch HR/Employee |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Native `<input type="date">` (D-05) | Custom RangePicker-Atom | RangePicker wäre wiederverwendbarer, aber Phase 8 ist **die einzige** Stelle in v1.3, die einen Range-Picker braucht — kein Reuse-Vorteil. D-05 explizit gegen Reusable. |
| Inline-Komponenten in `page/absences.rs` | Reusable Atoms in `component/atoms/` | UI-SPEC Komponenten-Inventory schreibt inline (Domain-spezifisch, Phase-8-only). Falls in Phase 9/10 wiederverwendet → spätere Extraktion. |
| Eigener `VacationBalanceTO` | `EmployeeReportTO` direkt verwenden | `EmployeeReportTO` ist groß (`vacation_days`, `vacation_carryover`, `vacation_entitlement` + alle anderen Felder). Aggregat-DTO macht REST-Surface schmaler & Frontend-State leichter. **Plan-Phase-Decision** (D-04) — Empfehlung: dedizierter `VacationBalanceTO`. |
| Coroutine-Service mit Action-Enum | `use_resource` direkt in der Page | Service-Pattern erlaubt Cross-Page-Refresh (`ABSENCE_REFRESH`-Bump aus Modal-Submit), Action-Enum erlaubt zentrales 409/422-Handling. Etabliert in `service/employee.rs`. |

**Installation:** Keine neuen Crate-Dependencies in Phase 8. Alle benötigten Libraries sind bereits in `shifty-dioxus/Cargo.toml` und `shifty-backend/Cargo.toml`.

**Version verification:** Nicht erforderlich — alle Libraries kommen aus dem bestehenden Workspace und sind in v1.2 Phase 6 (rest-types-Unification) bereits gegen WASM-Build verifiziert.

---

## Architecture Patterns

### System Architecture Diagram

```
                    ┌─────────────────────────────────────────────┐
                    │         Frontend (shifty-dioxus, WASM)      │
                    │                                             │
   Browser ──────►  │   ┌─Router::Absences ──► AbsencePage────┐   │
                    │   │                       │             │   │
                    │   │                       ▼             │   │
                    │   │   AUTH.read() ─► is_hr ─► variant   │   │
                    │   │                       │             │   │
                    │   │                       ▼             │   │
                    │   │   ABSENCE_STORE ◄── AbsenceList     │   │
                    │   │   ABSENCE_REFRESH ── AbsenceModal   │   │
                    │   │   VACATION_BALANCE_STORE            │   │
                    │   │                       │             │   │
                    │   │                       ▼             │   │
                    │   │   loader.rs (TO→state, side-joins)  │   │
                    │   │                       │             │   │
                    │   │                       ▼             │   │
                    │   │   api.rs (reqwest)                  │   │
                    │   └───────────────────────│─────────────┘   │
                    └───────────────────────────│─────────────────┘
                                                │ HTTP/JSON
                                                │ (Dioxus.toml proxy)
                                                ▼
        ┌──────────────────────────────────────────────────────────────────┐
        │                Backend (shifty-backend Workspace)                │
        │                                                                  │
        │   ┌─REST (rest/src/absence.rs) ──────────────────────────────┐   │
        │   │  /absence-period CRUD (existiert seit v1.0)              │   │
        │   │  + NEU: /vacation-balance/* (Plan-Phase)                 │   │
        │   └────────────────────────────│─────────────────────────────┘   │
        │                                ▼                                 │
        │   ┌─Service-Layer ────────────────────────────────────────────┐  │
        │   │  AbsenceService (BL-Tier)  ──► AbsenceDao                 │  │
        │   │      ├─► BookingService       (basic)                     │  │
        │   │      ├─► SalesPersonUnavail.  (basic)                     │  │
        │   │      ├─► SpecialDayService    (basic)                     │  │
        │   │      └─► WorkingHoursService  (basic)                     │  │
        │   │                                                           │  │
        │   │  NEU: VacationBalanceService (BL-Tier)                    │  │
        │   │      ├─► AbsenceService       (BL — Cross-Entity)         │  │
        │   │      │   ├─► or AbsenceDao    (Plan-Phase)                │  │
        │   │      ├─► WorkingHoursService  (basic — entitlement)       │  │
        │   │      ├─► CarryoverService     (basic — Vorjahr)           │  │
        │   │      ├─► PermissionService                                │  │
        │   │      ├─► SalesPersonService   (für hr ∨ self-Check)       │  │
        │   │      └─► TransactionDao                                   │  │
        │   └────────────────────────────│─────────────────────────────┘   │
        │                                ▼                                 │
        │                   SQLite (absence_period table)                  │
        └──────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

**Frontend additions:**

```
shifty-dioxus/src/
├── api.rs                       # +~7 fns: list_absence_periods, get_absence_period,
│                                #         create_absence_period, update_absence_period,
│                                #         delete_absence_period, list_absence_periods_by_sales_person,
│                                #         get_vacation_balance (or shape per Plan-Phase)
├── loader.rs                    # +load_absence_periods (with SalesPerson-cross-resolve for HR),
│                                #  +load_vacation_balance (Self),
│                                #  +load_team_vacation (HR aggregate)
├── service/
│   ├── absence.rs               # NEW: AbsenceAction enum + ABSENCE_STORE + ABSENCE_REFRESH
│   ├── vacation_balance.rs      # NEW: VACATION_BALANCE_STORE + VACATION_TEAM_STORE
│   └── mod.rs                   # +pub mod absence; +pub mod vacation_balance;
├── state/
│   └── absence_period.rs        # NEW: AbsencePeriod state-type with From<&AbsencePeriodTO>
├── page/
│   ├── absences.rs              # NEW: AbsencePage + AbsenceModal + WarningList +
│   │                            #      CategoryBadge + StatusPill + VacationEntitlementCard +
│   │                            #      VacationPerPersonList + AbsenceList + AbsenceFilterBar +
│   │                            #      StatsGrid + Banner-Helpers (inline per UI-SPEC)
│   └── mod.rs                   # +pub mod absences; +pub use absences::AbsencesPage;
├── i18n/
│   ├── mod.rs                   # ~50 neue Key-Variants unter "// Absence management"
│   ├── en.rs                    # alle Keys befüllt (UI-SPEC Copywriting Tabelle)
│   ├── de.rs                    # alle Keys befüllt
│   └── cs.rs                    # alle Keys befüllt
├── component/
│   └── top_bar.rs               # +NavTarget::Absences, +visibility.absences (unconditional true)
└── router.rs                    # +Route::Absences  ─► AbsencesPage
```

**Backend additions:**

```
shifty-backend/
├── service/src/
│   ├── vacation_balance.rs      # NEW: VacationBalance struct + VacationBalanceService trait
│   └── lib.rs                   # +pub mod vacation_balance;
├── service_impl/src/
│   └── vacation_balance.rs      # NEW: VacationBalanceServiceImpl via gen_service_impl!
├── rest/src/
│   ├── vacation_balance.rs      # NEW: GET /vacation-balance/{sales_person_id}/{year} +
│   │                            #      ggf. GET /vacation-balance/team/{year} (HR-Aggregat)
│   └── lib.rs                   # +mod vacation_balance; +nest("/vacation-balance", ...)
│                                # +ApiDoc nest-entry
├── rest-types/src/
│   └── lib.rs                   # +VacationBalanceTO struct (entitled/used/planned/remaining)
│                                # +Optional VacationTeamRowTO (Per-Person für HR)
├── shifty_bin/src/
│   └── main.rs                  # +VacationBalanceServiceDependencies impl
│                                # +VacationBalanceService type alias
│                                # +Konstruktor (NACH AbsenceService + WorkingHoursService + CarryoverService)
│                                # +RestStateImpl-Wiring (struct field + getter)
└── shifty-dioxus/Dioxus.toml    # +[[web.proxy]] backend = "http://localhost:3000/vacation-balance"
                                  # +[[web.proxy]] backend = "http://localhost:3000/absence-period"  ← NEW (Phase 8 needs it!)
```

### Pattern 1: Page-Service-State-Coroutine (Frontend)

**What:** Gleiche Struktur wie `service/employee.rs` + `page/employee_details.rs`.

**When to use:** Page mit CRUD + cross-page-refresh-Bedarf + zentralem 409/422-Handling.

**Example:**
```rust
// src/service/absence.rs (NEW) — analog `service/employee.rs:46-86`

use dioxus::prelude::*;
use std::rc::Rc;
use uuid::Uuid;
use rest_types::AbsencePeriodTO;
use crate::state::absence_period::AbsencePeriod;

pub static ABSENCE_STORE: GlobalSignal<Rc<[AbsencePeriod]>> = Signal::global(|| Rc::new([]));
pub static ABSENCE_REFRESH: GlobalSignal<u64> = Signal::global(|| 0);

#[derive(Debug)]
pub enum AbsenceAction {
    LoadAll,                                  // HR
    LoadForSalesPerson(Uuid),                 // Employee
    Create(AbsencePeriodTO),
    Update(AbsencePeriodTO),
    Delete(Uuid),
    Refresh,
}

pub async fn absence_service(mut rx: UnboundedReceiver<AbsenceAction>) {
    use futures_util::StreamExt;
    while let Some(action) = rx.next().await {
        // analog `employee_service` in src/service/employee.rs:191-276
        // bei Erfolg: bump_absence_refresh()
        // bei 409: ShiftyError::Conflict → Banner-State im Page (per-Modal, NICHT global)
    }
}
```

```rust
// src/app.rs — +use_coroutine(service::absence::absence_service);
// src/app.rs — +use_coroutine(service::vacation_balance::vacation_balance_service);
```

```rust
// src/page/absences.rs — Coroutine-Konsument analog page/employee_details.rs:53
let absence_service = use_coroutine_handle::<AbsenceAction>();
let auth = AUTH.read().clone();
let is_hr = auth.auth_info.as_ref().map(|a| a.has_privilege("hr")).unwrap_or(false);
```

Source: `service/employee.rs:46-86`, `page/employee_details.rs:53-105`, `app.rs:13-26`.

### Pattern 2: Cross-Source-Loader mit Side-Joins

**What:** TOs aus mehreren Endpoints zu State-Items mit Cross-Resolve aggregieren (z. B. `Booking → SalesPerson.name` für Anzeige).

**When to use:** HR-Liste muss Absence + zugehörigen Sales-Person-Namen zusammen anzeigen.

**Example:** `loader.rs:76-102` (`load_bookings`) macht es: lädt `BookingTO`, mapt zu `Booking`, joint mit `Rc<[SalesPerson]>` für `label` + `background_color`.

```rust
// src/loader.rs — NEW load_absence_periods analog load_bookings
pub async fn load_absence_periods(
    config: Config,
    sales_persons: Rc<[SalesPerson]>,  // already loaded
) -> Result<Rc<[AbsencePeriod]>, ShiftyError> {
    let tos = api::list_absence_periods(config).await?;
    let absences: Rc<[AbsencePeriod]> = tos
        .iter()
        .map(|to| {
            let mut a = AbsencePeriod::from(to);
            if let Some(sp) = sales_persons.iter().find(|sp| sp.id == a.sales_person_id) {
                a.person_name = sp.name.clone();
                a.background_color = sp.background_color.clone();
            }
            a
        })
        .collect();
    Ok(absences)
}
```

Source: `loader.rs:76-102`.

### Pattern 3: Optimistic-Lock + 409-Handling (Frontend)

**What:** PUT-Call → bei `StatusCode::CONFLICT` → `ShiftyError::Conflict(message)` → Page-State setzt Banner + behält Form-Inhalt.

**When to use:** Jeder PUT mit `$version`-Field.

**Example:** `api.rs:452-468` (`update_extra_hour`):
```rust
pub async fn update_extra_hour(config: Config, extra_hours: ExtraHoursTO) -> Result<ExtraHoursTO, ShiftyError> {
    let url = format!("{}/extra-hours/{}", config.backend, extra_hours.id);
    let response = client.put(url).json(&extra_hours).send().await?;
    if response.status() == reqwest::StatusCode::CONFLICT {
        return Err(ShiftyError::Conflict(String::new()));
    }
    response.error_for_status_ref()?;
    let updated: ExtraHoursTO = response.json().await?;
    Ok(updated)
}
```

D-08 unterscheidet sich hier: statt globalem `ERROR_STORE` (wie `service/employee.rs:208-218`) → Modal-lokaler Banner mit Reload-Button.

Source: `api.rs:452-468`, `error.rs:11-15`, `service/employee.rs:206-220`.

### Pattern 4: Wrapper-Result-DTO mit Forward-Warnings (Backend)

**What:** POST/PUT-Response trägt sowohl die persistierte Entity als auch `warnings: Vec<WarningTO>`.

**Already established:** `AbsencePeriodCreateResultTO { absence: AbsencePeriodTO, warnings: Vec<WarningTO> }` (`rest-types/src/lib.rs:1862-1866`).

**Status code:** `201` (POST) bzw. `200` (PUT) **mit Body** — Warnings sind Erfolgs-Pfad, kein Error.

Source: `rest-types/src/lib.rs:1860-1876`, `rest/src/absence.rs:67-77` (POST), 165-175 (PUT).

### Pattern 5: gen_service_impl! + Tier-Konvention (Backend)

**What:** Service-Impl mit deklarativen Dependencies via Macro.

**Example:** `service_impl/src/absence.rs:45-63` zeigt den **erweiterten** Dep-Set (BL-Tier mit `BookingService`, `SalesPersonUnavailableService`, `SlotService` als Cross-Entity-Konsum):

```rust
gen_service_impl! {
    struct AbsenceServiceImpl: AbsenceService = AbsenceServiceDeps {
        AbsenceDao: AbsenceDao<Transaction = Self::Transaction> = absence_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        SalesPersonService: SalesPersonService<...> = sales_person_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        SpecialDayService: SpecialDayService<...> = special_day_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<...> = employee_work_details_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
        BookingService: BookingService<...> = booking_service,
        SalesPersonUnavailableService: SalesPersonUnavailableService<...> = sales_person_unavailable_service,
        SlotService: SlotService<...> = slot_service,
    }
}
```

**Empfohlene Deps für `VacationBalanceServiceImpl` (BL-Tier, `[ASSUMED]` für genaue Zusammensetzung):**

```rust
gen_service_impl! {
    struct VacationBalanceServiceImpl: VacationBalanceService = VacationBalanceServiceDeps {
        AbsenceService: AbsenceService<...> = absence_service,                  // BL — used Vacation-Tage
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<...> = working_hours_service, // entitlement
        CarryoverService: CarryoverService<...> = carryover_service,            // Vorjahres-Übertrag
        SalesPersonService: SalesPersonService<...> = sales_person_service,     // hr ∨ self-Check + HR-Liste
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,                              // current year if no path-arg
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}
```

**Tier-Validation:** AbsenceService ist bereits BL-Tier (es konsumiert mehrere Basic-Services). VacationBalanceService konsumiert AbsenceService → es ist auch BL (BL→BL ist erlaubt, kein Cycle, weil AbsenceService nichts von VacationBalanceService weiß) — analog zu CarryoverRebuildService (`main.rs:482-493`, BL→ReportingService).

**DI-Konstruktion in `main.rs`:** VacationBalanceService MUSS nach AbsenceService (Z. 798), WorkingHoursService (Z. 788), CarryoverService konstruiert werden. Konkrete Reihenfolge — Plan-Phase legt Position fest.

Source: `service_impl/src/absence.rs:45-63`, `shifty_bin/src/main.rs:228-255` (Deps-impl) und `:798-815` (Konstruktion).

### Pattern 6: REST-Layer Dünn-Wrapper mit error_handler

**What:** REST-Handler ist nur Body-Parse + Service-Call + DTO-Conversion + Error-Mapping.

**Example:** `rest/src/absence.rs:55-77` (POST):
```rust
pub async fn create_absence_period<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(body): Json<AbsencePeriodTO>,
) -> Response {
    error_handler((async {
        let svc = rest_state.absence_service();
        let result = svc.create(&(&body).into(), context.into(), None).await?;
        let to = AbsencePeriodCreateResultTO::from(&result);
        Ok(Response::builder().status(201).header(...).body(...).unwrap())
    }).await)
}
```

`error_handler` mappt automatisch:
- `ServiceError::Forbidden` → `403`
- `ServiceError::EntityNotFound(_)` → `404`
- `ServiceError::EntityConflicts(_,_,_)` → `409` (für Version-Konflikt-Fall)
- `ServiceError::ValidationError(_)` → `422` (für Self-Overlap)
- `ServiceError::DateOrderWrong(_,_)` → `422`

Source: `rest/src/lib.rs:122-267`.

### Anti-Patterns to Avoid

- **`window.confirm` für Delete-Confirmation** (Mockup-Tweak) — D-07 verbietet; nutze `Dialog`-Komponente mit `DialogVariant::Center` und Width ~360 für Confirm-Variante.
- **`viewAs`-Toggle für HR/Employee** (Mockup-Tweak) — D-09 verbietet; AUTH-Context entscheidet.
- **Globaler Toast für 422 Self-Overlap** — D-11 verbietet; Inline-Banner unter Range-Feldern im Modal.
- **Silent-Refresh bei 409** — D-08 verbietet; explizit Banner + Reload-Button + User-Klick erforderlich, Form-State NICHT verwerfen.
- **Hand-Roll-Date-Range-Picker als neues Atom** — D-05 verbietet; native `<input type="date">` × 2 + Cross-Field-Validation in Page-Code.
- **`[ASSUMED] Custom Tailwind-Klassen mit `format!` in safelist NICHT eingetragen** — UI-SPEC `tailwind.config.js:7` Hinweis: Kategorie-Farb-Mapping MUSS via statische `match`-Arme in den Build kommen (Tailwind purge sieht sonst dynamische Klassen nicht).
- **AUTH-Read ohne `loading_done`-Check** — `AuthStore.loading_done == false` während Initial-Fetch (`service/auth.rs:9-42`); Page sollte `is_hr` defaulten erst NACH `loading_done == true` evaluieren, sonst flackert die UI.
- **`AbsencePeriodTO.id` non-nil bei POST** — `create()` enforced `!entity.id.is_nil()` → `ServiceError::IdSetOnCreate` → 422 (`service_impl/src/absence.rs:177`); Frontend MUSS `Uuid::nil()` setzen bei POST (analog `add_extra_hour` `id: Uuid::nil()` in `api.rs:407`).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Modal-Wrapper | Custom modal mit Backdrop+ESC | `component/dialog.rs` `Dialog` (Center) | 30 Tests, Body-Scroll-Lock, Focus-Trap, ESC-Handler bereits gebaut |
| Form-Field-Layout | Custom label/input/error-stack | `component/form/field.rs` `Field` mit `error: Option<ImStr>` | Span-Prop für Grid, Hint-vs-Error-Mutex |
| Date-Input | JS-Custom-Picker | `TextInput { input_type: "date" }` | Native HTML5, lokalisiert vom Browser |
| Select-Dropdown | Custom open/close | `SelectInput` mit Custom-Chevron-Background | Keyboard-Nav, native ARIA |
| Range-Picker | RangePicker-Atom | 2 × `TextInput { input_type="date" }` + Cross-Field-Validate (D-05) | Reuse-Bedarf in v1.3 = 1 → kein Atom |
| Confirm-Dialog | window.confirm | `Dialog` Center-Variante Width 360 (D-07) | Konsistente Theming |
| HR/Employee-Switch | UI-Toggle | `AUTH.read().auth_info.has_privilege("hr")` (D-09) | Auth-Context ist Quelle |
| 409-Conflict-Detection | Custom-Status-Parsing | `reqwest::StatusCode::CONFLICT` → `ShiftyError::Conflict` | Bereits in `api.rs:452-468` etabliert |
| Forward-Warnings-Wrapper-DTO | Custom HTTP-Response-Parsing | `AbsencePeriodCreateResultTO` aus `rest-types` | Bereits in v1.0 Phase 3 etabliert |
| Self-Overlap-Logic im Frontend | Pre-Check-Helpers | Backend `find_overlapping` + 422 ServiceError | Backend ist Source of Truth (D-11) |
| Resturlaubs-Berechnung im Frontend | Subtraktion in JS | Backend-Endpoint mit `WorkingHoursService.vacation_days_for_year` + `CarryoverService.get_carryover` (D-03) | Backend hat bereits die Berechnung in `service_impl/reporting.rs:586-631` |
| OpenAPI-Annotations | Manuelle Schema-Beschreibung | `#[utoipa::path]` + `ToSchema` Derive | CLAUDE.md Pflicht; Snapshot-Test in v1.0 Phase 4 etabliert |
| `gen_service_impl!`-Wiring | Manueller `impl Service for X` | Macro-Expansion (`service_impl/src/macros.rs`) | CLAUDE.md Pflicht |
| Service-Tier-Klassifizierung | Eigene Heuristik | `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen" | Doku ist verbindlich; Cycles vermeiden |

**Key insight:** Die Frontend-Code-Basis ist bereits sehr reich an wiederverwendbaren Atoms — Phase 8 ist **eine Komposition**, kein Net-New-Components-Build. Der Backend-Endpoint ist der einzige Net-New-Code; das Pattern dafür (Service-Trait + Service-Impl mit `gen_service_impl!` + REST-Wrapper + Tier-Konvention) ist 1:1 aus `AbsenceService` übernehmbar.

---

## Common Pitfalls

### Pitfall 1: rest-types Cross-Crate `service-impl`-Feature-Pull-In

**What goes wrong:** Frontend-Cargo zieht `service-impl`-Feature → das Backend-`service`-Crate kommt in den WASM-Build → Toolchain bricht.

**Why it happens:** `rest-types/Cargo.toml` definiert ein `service-impl`-Feature, das auf `service` zeigt; ohne explizites `default-features = false` wird das Default-Feature-Set inkludiert.

**How to avoid:** Frontend-`shifty-dioxus/Cargo.toml` MUSS `default-features = false` für `rest-types` haben (Z. 28-30, bereits gesetzt). Plan darf diesen Eintrag NICHT entfernen.

**Warning signs:** `cargo build --target wasm32-unknown-unknown` schlägt fehl mit Errors aus dem `service`-Crate (z. B. tokio-Features, sqlx).

**Source:** STATE.md "Constraints In Force"; v1.2 Phase 6 D-Phase6-XX.

### Pitfall 2: `Locale::En`-statt-`Locale::De`-Bug

**What goes wrong:** In `de.rs` wird versehentlich `i18n.add_text(Locale::En, Key::Foo, "deutscher text")` statt `Locale::De` geschrieben → deutsche Texte werden nie ausgegeben (kommen nur über Fallback aus En).

**Why it happens:** Copy-Paste vom `en.rs` zu `de.rs` ohne Suchen-Ersetzen.

**How to avoid:** Beim Schreiben jedes Locale-Files **als ersten Schritt** alle `Locale::*`-Aufrufe mit Editor-Suche prüfen. UI-SPEC § "i18n Compliance" macht dies zur Sign-Off-Bedingung.

**Warning signs:** Cs-Locale-Test (UI-SPEC schreibt vor: alle drei Locales in `i18n_employees_keys_match_german_reference`-Pattern testen) zeigt fehlende Übersetzung.

**Source:** STATE.md "Constraints In Force", `shifty-dioxus/CLAUDE.md` § "Common Issues 2", historischer Bug behoben.

### Pitfall 3: Service-Tier-Konvention-Verstoß

**What goes wrong:** Plan macht `VacationBalanceService` zum Basic-Tier-Service mit AbsenceService-Dependency → AbsenceService ist BL → Basic konsumiert BL → zyklische DI bei Konstruktor-Reihenfolge.

**Why it happens:** Falsche Klassifizierung — Service-Tier-Konvention sieht "Basic = nur DAOs/Permission/Tx".

**How to avoid:** Plan MUSS `VacationBalanceService` als **Business-Logic-Tier** klassifizieren (D-04). DI-Konstruktion in `main.rs` NACH AbsenceService (Z. 798), WorkingHoursService (Z. 788), CarryoverService (Z. 843).

**Warning signs:** Compiler-Error in `main.rs` "use of moved value" oder "value of type ... cannot be constructed because deps are not in scope yet".

**Source:** `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen"; v1.1 Phase 5 D-12-Override-Präzedenz.

### Pitfall 4: AUTH-State während Loading

**What goes wrong:** Page evaluiert `is_hr = AUTH.read().auth_info.has_privilege("hr")` direkt — vor Initial-Auth-Fetch returnt es `false` → Page rendert kurzzeitig im Employee-Modus, dann switcht auf HR → Flackern + falsche initiale API-Calls.

**Why it happens:** `AuthStore.auth_info` ist `None` während `loading_done == false`.

**How to avoid:** Page-Render-Branch früh: `if !auth.loading_done { return rsx! { "Loading..." }; }`. Pattern aus `auth.rs:15-23`.

**Warning signs:** UAT-Test sieht zwei API-Calls (`by-sales-person/...` zuerst, dann `/absence-period`).

**Source:** `auth.rs:15-23`, `service/auth.rs:9-42`.

### Pitfall 5: Tailwind Purge eliminiert Kategorie-Farb-Klassen

**What goes wrong:** Code generiert Class-Strings dynamisch, z. B. `format!("text-{} bg-{}-soft", category)` → Tailwind-Build sieht keine literalen Klassennamen → Klassen werden gepurged → Production-Build zeigt schwarz/weiß.

**Why it happens:** Tailwind v3 scannt Source nach String-Literalen.

**How to avoid:** Statische `match`-Arme:
```rust
let (text, bg) = match category {
    AbsenceCategoryTO::Vacation    => ("text-good",      "bg-good-soft"),
    AbsenceCategoryTO::SickLeave   => ("text-warn",      "bg-warn-soft"),
    AbsenceCategoryTO::UnpaidLeave => ("text-ink-muted", "bg-surface-2"),
};
```

**Warning signs:** Produktions-Build hat farblose Pills, Dev-Build (`npx tailwindcss --watch` mit safelist) hat farbige.

**Source:** UI-SPEC § "Color" — Kategorie-Farb-Mapping; `tailwind.config.js:7` Hinweis.

### Pitfall 6: jj-Repo + git-commit aus Agent

**What goes wrong:** Plan-Executor ruft `git commit` direkt auf → kollidiert mit jj-Co-Located-Tracking, hinterlässt orphaned change.

**Why it happens:** Standard-Git-Workflow-Reflexe.

**How to avoid:** GSD `commit_docs: false` (verifiziert in `.planning/config.json` — keine Auto-Commits). User committed manuell mit `jj`. CLAUDE.local.md macht dies explizit.

**Warning signs:** `jj st` zeigt unsaubere Konfliktstände nach Agent-Run.

**Source:** `.planning/config.json`, CLAUDE.local.md.

### Pitfall 7: Forward-Warnings-Modal-Flow vs. Modal-Close

**What goes wrong:** POST liefert 201 mit `warnings.len() > 0` → Page schließt Modal sofort → User sieht Warnings nie.

**Why it happens:** Naïves Refactoring von "Close on success" ohne Warning-Branch.

**How to avoid:** UI-SPEC Interaction-Contract `Forward-Warnings-Flow` Schritt 1-5 verbindlich: Warnings-Liste rendern, Submit-Btn-Beschriftung wechselt zu "Verstanden" (`AbsenceWarningAcknowledgeBtn`), Klick auf "Verstanden" schließt Modal.

**Warning signs:** UAT verlangt Warning-Sichtbarkeit; visueller Test schlägt fehl.

**Source:** UI-SPEC § "Interaction Contract" → "Forward-Warnings-Flow", D-12.

### Pitfall 8: Status-Berechnung ohne Locale-Stable Date

**What goes wrong:** Status `Aktiv/Geplant/Beendet` nutzt `chrono::Local::now()` → in WASM/SSR-Tests undeterministisch → Test flaky.

**Why it happens:** `Local::now` zieht Browser-Zeitzone und tatsächliche Uhrzeit.

**How to avoid:** Pure-Function `compute_status(from: Date, to: Date, today: Date) -> AbsenceStatus` exposen, `today` als Parameter — testbar mit fixed dates wie `extra_hours_modal.rs:54-58` `current_datetime_for_init` mit `#[cfg(not(target_arch = "wasm32"))]`-Branch.

**Warning signs:** Frontend-Tests schlagen je nach Build-Zeit fehl.

**Source:** UI-SPEC § "Status-Berechnung"; `extra_hours_modal.rs:50-58` Pattern.

### Pitfall 9: Frontend-WASM-Build ohne `nix develop`

**What goes wrong:** `cargo build --target wasm32-unknown-unknown` schlägt fehl, weil die WASM-Toolchain nicht im PATH ist.

**Why it happens:** NixOS-default-shell hat den `wasm32-unknown-unknown`-Target nicht.

**How to avoid:** `nix develop` (NICHT `nix-shell`, MEMORY.md `feedback_no_unauthorized_install`). Für SDK/sqlx-Befehle gleichermaßen.

**Warning signs:** `error[E0463]: can't find crate for std` mit `--target wasm32-unknown-unknown`.

**Source:** `shifty-backend/CLAUDE.local.md`, MEMORY.md `reference_local_dev_commands`.

### Pitfall 10: Dx-Proxy fehlt für `/absence-period`

**What goes wrong:** Frontend ruft `/absence-period` an, aber `Dioxus.toml` hat keinen Eintrag → Dev-Server (Port 8080) schickt nicht zu Backend (Port 3000) → 404.

**Why it happens:** `/absence-period` ist nicht im aktuellen `Dioxus.toml` (verifiziert: 25 Einträge, `/absence-period` NICHT dabei).

**How to avoid:** Plan MUSS `Dioxus.toml` ergänzen mit:
```toml
[[web.proxy]]
backend = "http://localhost:3000/absence-period"
[[web.proxy]]
backend = "http://localhost:3000/vacation-balance"  # oder gewählter Endpoint-Name
```

**Warning signs:** Dev-Server liefert 404 für die neuen Endpoints.

**Source:** `shifty-dioxus/Dioxus.toml`, gegrept gegen `absence-period` und `vacation-balance`.

---

## Code Examples

### Example 1: Page mit Auth-Gate + Coroutine-Service-Konsum

```rust
// src/page/absences.rs (NEW) — analog page/employee_details.rs:39-130
use dioxus::prelude::*;
use crate::{
    component::{TopBar, error_view::ErrorView},
    service::{absence::{AbsenceAction, ABSENCE_STORE}, auth::AUTH, i18n::I18N},
};

#[component]
pub fn AbsencesPage() -> Element {
    let auth = AUTH.read().clone();
    if !auth.loading_done {
        return rsx! { div { "Loading..." } };
    }
    let is_hr = auth.auth_info.as_ref().map(|a| a.has_privilege("hr")).unwrap_or(false);

    let absence_service = use_coroutine_handle::<AbsenceAction>();
    use_effect(move || {
        if is_hr {
            absence_service.send(AbsenceAction::LoadAll);
        } else if let Some(sp_id) = auth.auth_info.as_ref().and_then(|_| /* current sales_person_id */ None::<uuid::Uuid>) {
            absence_service.send(AbsenceAction::LoadForSalesPerson(sp_id));
        }
    });

    rsx! {
        TopBar {}
        ErrorView {}
        // ... Header + VacationEntitlementCard + StatsGrid + FilterBar + AbsenceList
    }
}
```

Source: `page/employee_details.rs:39-130`, `state/auth_info.rs:24-26`, `service/auth.rs:50`.

### Example 2: API-Layer für `/absence-period` CRUD

```rust
// src/api.rs (additions) — analog api.rs:392-468 (extra_hours)
use rest_types::{AbsencePeriodTO, AbsencePeriodCreateResultTO};
use crate::error::ShiftyError;

pub async fn list_absence_periods(config: Config)
    -> Result<Rc<[AbsencePeriodTO]>, reqwest::Error> {
    let url = format!("{}/absence-period", config.backend);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    Ok(res)
}

pub async fn list_absence_periods_by_sales_person(
    config: Config, sales_person_id: Uuid,
) -> Result<Rc<[AbsencePeriodTO]>, reqwest::Error> {
    let url = format!("{}/absence-period/by-sales-person/{}", config.backend, sales_person_id);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    Ok(response.json().await?)
}

pub async fn create_absence_period(
    config: Config, body: AbsencePeriodTO,
) -> Result<AbsencePeriodCreateResultTO, ShiftyError> {
    let url = format!("{}/absence-period", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&body).send().await?;
    if response.status() == reqwest::StatusCode::UNPROCESSABLE_ENTITY {
        // 422 — Self-Overlap or DateOrderWrong
        let text = response.text().await.unwrap_or_default();
        return Err(ShiftyError::Validation(text));  // NEU: Variant in ShiftyError
    }
    response.error_for_status_ref()?;
    let result: AbsencePeriodCreateResultTO = response.json().await?;
    Ok(result)
}

pub async fn update_absence_period(
    config: Config, id: Uuid, body: AbsencePeriodTO,
) -> Result<AbsencePeriodCreateResultTO, ShiftyError> {
    let url = format!("{}/absence-period/{}", config.backend, id);
    let client = reqwest::Client::new();
    let response = client.put(url).json(&body).send().await?;
    if response.status() == reqwest::StatusCode::CONFLICT {
        return Err(ShiftyError::Conflict(String::new()));
    }
    if response.status() == reqwest::StatusCode::UNPROCESSABLE_ENTITY {
        let text = response.text().await.unwrap_or_default();
        return Err(ShiftyError::Validation(text));
    }
    response.error_for_status_ref()?;
    Ok(response.json().await?)
}

pub async fn delete_absence_period(
    config: Config, id: Uuid,
) -> Result<(), reqwest::Error> {
    let url = format!("{}/absence-period/{}", config.backend, id);
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    Ok(())
}
```

Note: `ShiftyError::Validation(String)` ist ein **NEW Variant** — Plan-Phase entscheidet ob hinzufügen (für 422-Inline-Anzeige nötig, D-11) oder ob als `Conflict`-Variant gemultiplext.

Source: `api.rs:392-468`, `error.rs:1-15`.

### Example 3: Backend Resturlaubs-Service Trait

```rust
// service/src/vacation_balance.rs (NEW)
use std::sync::Arc;
use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;
use crate::{permission::Authentication, ServiceError};

#[derive(Clone, Debug, PartialEq)]
pub struct VacationBalance {
    pub sales_person_id: Uuid,
    pub year: u32,
    pub entitled_days: f32,        // from WorkingHoursService.vacation_days_for_year
    pub carryover_days: i32,       // from CarryoverService.get_carryover().vacation
    pub used_days: f32,            // sum of past Vacation AbsencePeriods (or via ReportingService.vacation_days)
    pub planned_days: f32,         // sum of future Vacation AbsencePeriods
    pub remaining_days: f32,       // entitled + carryover − (used + planned)
}

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait VacationBalanceService {
    type Context: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// HR ∨ self
    async fn get(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<VacationBalance, ServiceError>;

    /// HR only — Aggregate for VacationPerPersonList
    async fn get_team(
        &self,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[VacationBalance]>, ServiceError>;
}
```

Source: Strukturanalogie zu `service/src/absence.rs:138-216`, `service/src/carryover.rs:51-69`.

### Example 4: i18n-Key-Block einfügen

```rust
// src/i18n/mod.rs — neuer Block am Ende (vor Z. 406)
    // Absence management
    AbsencePageTitle,
    AbsencePageSubtitle,
    AbsenceMenuLabel,
    AbsenceNewBtn,
    // ... ca. 50 weitere Keys per UI-SPEC § "Copywriting Contract"
```

```rust
// src/i18n/de.rs — Beispiel-Block (analog Z. 18-24)
    // Absence management (Phase 8)
    i18n.add_text(Locale::De, Key::AbsencePageTitle, "Abwesenheiten");
    i18n.add_text(Locale::De, Key::AbsencePageSubtitle,
        "Urlaub, Krankheit und unbezahlte Freistellung als Zeiträume. \
         Stunden pro Tag werden aus dem gültigen Arbeitsvertrag abgeleitet.");
    // ...
```

⚠ **Pitfall 2 (Locale::En-statt-De):** Bei Copy-Paste alle `Locale::De`-Aufrufe in `de.rs` editor-suchen und prüfen.

Source: `i18n/de.rs:1-25`, UI-SPEC § "Copywriting Contract".

### Example 5: TopBar Menü-Eintrag hinzufügen

```rust
// src/component/top_bar.rs — Änderungen

// Z. 21-30: NavVisibility — Feld hinzufügen
pub(crate) struct NavVisibility {
    pub shiftplan: bool,
    pub my_shifts: bool,
    pub my_time: bool,
    pub year_overview: bool,
    pub absences: bool,            // NEU (D-10: für alle Eingeloggten)
    pub employees: bool,
    pub billing_periods: bool,
    pub user_management: bool,
    pub templates: bool,
}

// Z. 32-45: nav_visibility — `absences: true` für alle
pub(crate) fn nav_visibility(auth_info: Option<&AuthInfo>, is_paid: bool) -> NavVisibility {
    let has = |p: &str| auth_info.map(|a| a.has_privilege(p)).unwrap_or(false);
    let show_reports = has("hr");
    let logged_in = auth_info.is_some();
    NavVisibility {
        shiftplan: has("sales") || has("shiftplanner"),
        my_shifts: has("sales"),
        my_time: is_paid && !show_reports,
        year_overview: has("shiftplanner") || has("sales"),
        absences: logged_in,        // D-10
        employees: show_reports,
        billing_periods: show_reports,
        user_management: has("admin"),
        templates: has("admin"),
    }
}

// Z. 47-57: NavTarget — Variant hinzufügen
pub(crate) enum NavTarget {
    Shiftplan, MyShifts, MyTime, YearOverview,
    Absences,                      // NEU
    Employees, BillingPeriods, UserManagement, Templates,
}

// Z. 59-82: is_active_for — match-Arm
NavTarget::Absences => matches!(route, Route::Absences {}),

// Z. 312-371: nav_items — Eintrag einfügen
if visibility.absences {
    items.push((
        NavTarget::Absences,
        Route::Absences {},
        i18n.t(Key::AbsenceMenuLabel).to_string(),
    ));
}
```

Source: `component/top_bar.rs:21-82, 312-371`.

### Example 6: Router-Variant hinzufügen

```rust
// src/router.rs — Z. 19-51, neuer Variant
pub use crate::page::AbsencesPage;     // +pub use line

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
    // ...
    #[route("/absences/")]
    Absences {},                       // NEU
    // ...
}
```

Source: `router.rs:19-51`.

### Example 7: Backend Test-Pattern für `/vacation-balance`

```rust
// service_impl/src/test/vacation_balance.rs (NEW)
// Pattern: 1052 LOC `service_impl/src/test/absence.rs` mit ~30 Tests

#[tokio::test]
async fn get_returns_entitlement_minus_used_minus_planned() {
    let mut deps = MockVacationBalanceServiceDeps::new();
    // mock AbsenceService.find_by_sales_person → 2 Vacation periods (1 past, 1 future)
    // mock WorkingHoursService.find_by_sales_person_id → 1 contract w/ 25 vacation_days
    // mock CarryoverService.get_carryover → Some(Carryover { vacation: 5, ... })
    // mock PermissionService.check_permission(HR) → Forbidden
    // mock SalesPersonService.verify_user_is_sales_person → Ok(())
    let svc = VacationBalanceServiceImpl::new(deps);

    let result = svc.get(sales_person_id, 2026, auth, None).await.unwrap();

    assert_eq!(result.entitled_days, 25.0);
    assert_eq!(result.carryover_days, 5);
    assert_eq!(result.used_days, 5.0);     // 5 days vacation in past
    assert_eq!(result.planned_days, 10.0); // 10 days vacation in future
    assert_eq!(result.remaining_days, 25.0 + 5.0 - 5.0 - 10.0);
}

#[tokio::test]
async fn get_other_sales_person_without_hr_is_forbidden() { /* ... */ }

#[tokio::test]
async fn get_with_hr_succeeds() { /* ... */ }

#[tokio::test]
async fn get_team_without_hr_is_forbidden() { /* ... */ }
```

Source: `service_impl/src/test/absence.rs:256-565` Pattern.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `extra_hours` als Single-Day-Eintrag für Vacation/SickLeave/UnpaidLeave | Range-based `AbsencePeriod` | v1.0 (2026-05-03) | Phase 8 ist **die Frontend-Sichtbarkeit** dieser Domäne |
| Frontend-eigener `rest-types`-Fork | `rest-types` als single-source-of-truth, Cross-Workspace-Path-Dep mit `default-features = false` | v1.2 Phase 6 (2026-05-07) | Phase 8 baut auf dieser Foundation auf |
| Globale 409-Refresh aus `service/employee.rs:208-218` | Modal-lokaler 409-Banner mit User-Reload-Klick (D-08) | Phase 8 NEU (UX-Decision) | Form-State bleibt erhalten; User entscheidet |
| `window.confirm` für Delete | Dioxus-Dialog Center-Variante (D-07) | Phase 8 NEU | Konsistente Theming |
| Mockup-Tweak `viewAs` | Auth-Privilege-Driven (D-09) | Phase 8 NEU | Auth-Source-of-Truth |

**Deprecated/outdated:**
- `Notes for plan-phase` in `ROADMAP.md` § "Phase 8 Details" enthält `"Backend bleibt unangetastet"` — obsolet wegen D-02/D-03.
- `Phase 8 als reine Frontend-Phase` (Seed-Datei) — durch CONTEXT-Phase erweitert.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `VacationBalanceTO`-Shape `{ sales_person_id, year, entitled_days, carryover_days, used_days, planned_days, remaining_days }` | Architecture Patterns / Backend Code Example 3 | UI-SPEC § "VacationEntitlementCard" zeigt 5 Stats: Vertrag/Übertrag/Genommen/Beantragt/Verbleibend — also bestätigt durch i18n-Keys `VacationStatContract/Carryover/Used/Pending/Remaining`; Risk: Plan-Phase könnte andere Field-Namen wählen (z. B. `total_entitlement` statt `entitled_days`) — User-Confirmation in Plan-Phase nötig. |
| A2 | Resturlaubs-Endpoint = `GET /vacation-balance/{sales_person_id}/{year}` plus optional `GET /vacation-balance/team/{year}` für HR-Aggregat | Project Structure | D-03 sagt explizit "Plan-Phase entscheidet". Alternative: `GET /absence-period/vacation-balance/{sales_person_id}` (subnesting). Risk: niedrig — beide funktionieren; Plan-Phase muss entscheiden. |
| A3 | `VacationBalanceService` ist Business-Logic-Tier mit Deps `AbsenceService + WorkingHoursService + CarryoverService + SalesPersonService + PermissionService + ClockService + TransactionDao` | Pattern 5 | D-04 sagt "Erwartete Klassifizierung BL" und "Plan-Phase entscheidet". Alt-Variante: konsumiere `AbsenceDao` direkt statt `AbsenceService` → wäre weniger Tier-konform (BL sollte Service nutzen, nicht in den DAO-Layer absteigen). Risk: niedrig. |
| A4 | `used_days` (vergangene Vacation-Tage) wird über `AbsenceService.find_by_sales_person()` plus client-side-Filterung (`to_date < today`) gerechnet, NICHT über `ReportingService.get_report_for_employee().vacation_days` | Architecture Patterns | Risk: ReportingService liefert `vacation_days` bereits aufsummiert mit cap-logik (`reporting.rs:118-123`); für Vergangenheit wäre das die effizientere Quelle. Plan-Phase MUSS entscheiden. |
| A5 | `planned_days` (zukünftige Vacation-Tage) wird über `AbsenceService.find_by_sales_person()` + client-side-Filterung (`from_date > today`) plus `derive_hours_for_range`-Logik (Special-Days subtrahieren) gerechnet | Architecture Patterns | Risk: Wenn Plan-Phase entscheidet, Special-Days NICHT zu subtrahieren, weicht der Wert von der UI-Display-Logik ab. UI-SPEC § "AbsencePreviewFooter" sagt "Feiertage im Bereich werden mit 0 h verrechnet" — also Subtraktion erwartet. |
| A6 | i18n-Key-Anzahl: ca. 50 neue Keys (basierend auf UI-SPEC § "Copywriting Contract" — 11 Sektionen mit 3-15 Keys je) | Project Structure | Risk: niedrig — exakte Anzahl ergibt sich beim Schreiben. Wave-Planning kann es als grober Anker nutzen ("ein Plan-Item für i18n + Locale-Befüllung"). |
| A7 | Page kann mit ca. 800-1200 LOC Total realisiert werden (basis: `extra_hours_modal.rs` 597 LOC für nur Modal + `employee_details.rs` 213 LOC + extrapoliert auf Page+Modal+8 inline components) | Project Structure | Risk: niedrig — Plan-Phase verifiziert, ob Aufteilung in `page/absences.rs` + `component/absence_modal.rs` notwendig ist. UI-SPEC erlaubt beides. |
| A8 | Frontend nutzt Browser-aktuelles Datum für Status-Berechnung (`chrono::Local::now()` oder `time::OffsetDateTime::now_local()`) | Pitfall 8 | Risk: Wenn Plan-Phase Server-Zeit als Quelle wählt → zusätzlicher API-Call nötig; wahrscheinlich unnötig für "today"-Anchor. |
| A9 | Self-Overlap-422-Body enthält `OverlappingPeriod(uuid)` als ValidationFailureItem; Frontend muss Body parsen für Banner-Text | Pattern 6 / Pitfall | Source: `service_impl/src/absence.rs:204-205` — bestätigt. Risk: niedrig. Plan-Phase entscheidet, ob Body-Text User-facing übersetzbar gemacht wird (z. B. via Locale-Lookup mit lokalem `category`-String). |
| A10 | Frontend nutzt existing `loader.rs::load_sales_persons` als Cross-Resolve-Quelle für HR-Liste-Personen-Anzeige (Cross-Resolve analog `load_bookings`) | Pattern 2 | Source: `loader.rs:28-36, 76-102` — bestätigt. Risk: niedrig. |

---

## Runtime State Inventory

> Phase 8 ist **kein Rename/Refactor** — Greenfield-Erweiterung. Diese Sektion dient nur der Vollständigkeit.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — verified by grep against `dao_impl_sqlite/src/` for new schemas; AbsencePeriod-Schema ist bereits seit v1.0 Phase 1 vorhanden | none |
| Live service config | None — Phase 8 fügt keine neue OS-Service-Config hinzu | none |
| OS-registered state | None — kein systemd/Task-Scheduler-Eintrag involviert | none |
| Secrets/env vars | None — keine neuen Secrets nötig (gleicher Backend-Endpoint-Server) | none |
| Build artifacts | `cargo build --target wasm32-unknown-unknown` Artefakte werden refreshed; `dist/` für Tailwind ebenfalls | Standard-Rebuild reicht |

**Nothing found in any category** — Phase 8 ist ein additiver Build, keine Migration.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` workspace toolchain | Backend `cargo build/check/test` | (NixOS via `nix develop`) | rustc 1.x (vom flake.lock) | none |
| `wasm32-unknown-unknown` Target | Frontend `cargo build --target` | (NixOS via `nix develop`) | rustup-managed | none |
| `dx` (Dioxus CLI) | Frontend `dx serve --hot-reload` Dev-Workflow | (NixOS via `nix develop`) | 0.6.x | none |
| `npx tailwindcss` | Frontend `tailwind --watch` Dev-Workflow | (Node via NixOS-shell) | 3.x | none |
| `sqlx` CLI | Backend Migration falls neue Tables (Phase 8 fügt keine neue Table hinzu) | nicht erforderlich | — | — |
| Backend-Server (Port 3000) | UAT Smoke | Lokal startbar via `cargo run` | aktuelle Version | — |
| Frontend Dev-Server (Port 8080) | UAT Smoke | Lokal startbar via `dx serve` | — | — |

**Missing dependencies with no fallback:** none.

**Missing dependencies with fallback:** none.

**NixOS-Spezifika:** `nix develop` (NICHT `nix-shell` — `shell.nix` ist kaputt, MEMORY.md `reference_local_dev_commands`).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Backend Framework | Cargo + tokio + mockall (`#[automock]`) + `service_impl/src/test/`-Module |
| Frontend Framework | Cargo + dioxus-ssr 0.6 (für SSR-Component-Tests) + tokio-test + wasm-bindgen-test |
| Backend Config file | `Cargo.toml` workspace-root |
| Frontend Config file | `shifty-dioxus/Cargo.toml`, dev-dependencies Z. 77-81 |
| Quick run command (Backend) | `cargo test -p service_impl absence` (für absence-Tests) |
| Quick run command (Frontend) | `cargo test --manifest-path shifty-dioxus/Cargo.toml` |
| Full suite (Backend) | `cargo test --workspace` (466 Tests v1.2-Baseline) |
| Full suite (Frontend) | `cargo test --manifest-path shifty-dioxus/Cargo.toml` |
| WASM Compile-Gate | `cargo build --target wasm32-unknown-unknown --manifest-path shifty-dioxus/Cargo.toml` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FUI-A-01 | Route `/absences` erreichbar via Menü | Manual UAT-Smoke (Login + Click) | manuell | n/a |
| FUI-A-01 | Top-Bar zeigt "Abwesenheiten" für alle Eingeloggten | unit | `cargo test -p shifty-dioxus nav_visibility_includes_absences_for_logged_in_user` | ❌ Wave 0 |
| FUI-A-02 | HR-Sicht (privilege "hr") aktiviert Multi-Person-Filter | unit | `cargo test -p shifty-dioxus is_hr_branch_uses_load_all` | ❌ Wave 0 |
| FUI-A-02 | Employee-Sicht (kein "hr") nutzt by-sales-person | unit | gleicher Test | ❌ Wave 0 |
| FUI-A-03 | `Field` mit `error: Some(...)` rendert Validation-Error | unit (existing Test, Pattern erweitern) | `cargo test -p shifty-dioxus error_alone_renders_in_bad_colour` | ✅ `field.rs:140-153` |
| FUI-A-03 | `TextInput { input_type="date" }` rendert `<input type="date">` | unit (existing Test) | `cargo test -p shifty-dioxus text_input_custom_type_propagates` | ✅ `inputs.rs:237-249` |
| FUI-A-03 | Modal SSR-Test (open=true) rendert Range-Felder + Kategorie-Select | unit | `cargo test -p shifty-dioxus absence_modal_creates_renders_form_fields` | ❌ Wave 0 |
| FUI-A-03 | Backend liefert 422 bei Self-Overlap | integration | `cargo test -p service_impl test_create_self_overlap_same_category_returns_validation` | ✅ `service_impl/src/test/absence.rs:316` |
| FUI-A-04 | `WarningList` rendert mit `dense=true` 10px-Padding | unit (SSR-Test) | `cargo test -p shifty-dioxus warning_list_dense_uses_p_2_5` | ❌ Wave 0 |
| FUI-A-04 | Backend liefert `AbsencePeriodCreateResultTO` mit Forward-Warning bei booking-overlap | integration | `cargo test -p service_impl create_with_booking_overlap_emits_forward_warning` | ✅ existierende Tests in `absence.rs` |
| FUI-A-04 | i18n: AbsencePageTitle in alle 3 Locales gesetzt + nicht "??" | unit | `cargo test -p shifty-dioxus i18n_absence_keys_present_in_all_locales` | ❌ Wave 0 |
| Backend D-03/D-04 | VacationBalanceService.get returns entitlement − used − planned | unit (mockall) | `cargo test -p service_impl vacation_balance_get_subtracts_used_and_planned` | ❌ Wave 0 |
| Backend D-03/D-04 | VacationBalanceService.get_team forbidden für non-HR | unit | `cargo test -p service_impl vacation_balance_get_team_without_hr_is_forbidden` | ❌ Wave 0 |
| Backend D-03/D-04 | REST `/vacation-balance/{sp}/{year}` returns 200 with VacationBalanceTO | manual / curl-Smoke | `curl http://localhost:3000/vacation-balance/...` | manuell |
| Build-Gate | WASM compile success | smoke | `cd shifty-dioxus && nix develop --command cargo build --target wasm32-unknown-unknown` | ✅ Phase 7 baseline |
| Build-Gate | Backend cargo check + test workspace-wide grün | smoke | `cargo check --workspace && cargo test --workspace` | ✅ Phase 7 baseline |
| End-to-End | UAT: HR-Login + Employee-Login je einmal Anlage + Edit + Delete | manual UAT-smoke | UAT-Anweisung in PLAN.md V-Truth-Liste | manuell |

### Sampling Rate

- **Per task commit:** `cargo check --workspace` + relevanter modul-Test (`cargo test -p service_impl absence` oder `cargo test -p shifty-dioxus`).
- **Per wave merge:** Frontend WASM-Build-Gate + Backend `cargo test --workspace`.
- **Phase gate:** Full Backend + Frontend Tests grün, WASM-Build-Gate grün, manueller UAT-Smoke (HR + Employee Login) erfolgreich, dann `/gsd-verify-work`.

### Wave 0 Gaps

- [ ] `service/src/vacation_balance.rs` — Trait-Definition (existing absence-trait Pattern)
- [ ] `service_impl/src/vacation_balance.rs` — Impl via `gen_service_impl!`
- [ ] `service_impl/src/test/vacation_balance.rs` — ca. 8 Unit-Tests (analog `test/absence.rs:256-565`)
- [ ] `service_impl/src/test/mod.rs` — `pub mod vacation_balance;` ergänzen
- [ ] `rest/src/vacation_balance.rs` — REST-Wrapper + OpenAPI-Annotations
- [ ] `rest/src/lib.rs` — `mod vacation_balance;`, `nest("/vacation-balance", ...)`, ApiDoc nest-entry
- [ ] `rest-types/src/lib.rs` — `VacationBalanceTO` struct + (optional) `VacationTeamRowTO`
- [ ] `shifty_bin/src/main.rs` — `VacationBalanceServiceDependencies` impl, type-alias, Konstruktor, `RestStateImpl`-Field + getter
- [ ] `shifty-dioxus/src/state/absence_period.rs` — State-Type
- [ ] `shifty-dioxus/src/service/absence.rs` — Coroutine-Service
- [ ] `shifty-dioxus/src/service/vacation_balance.rs` — Coroutine-Service
- [ ] `shifty-dioxus/src/service/mod.rs` — `pub mod`s
- [ ] `shifty-dioxus/src/page/absences.rs` — Page mit allen 8 Inline-Components
- [ ] `shifty-dioxus/src/page/mod.rs` — `pub mod absences; pub use absences::AbsencesPage;`
- [ ] `shifty-dioxus/src/i18n/mod.rs` — ~50 Key-Variants
- [ ] `shifty-dioxus/src/i18n/{en,de,cs}.rs` — `add_text` für jeden Key, alle drei Locales
- [ ] `shifty-dioxus/src/router.rs` — `Route::Absences {}` Variant + `pub use AbsencesPage`
- [ ] `shifty-dioxus/src/component/top_bar.rs` — `NavTarget::Absences`, `NavVisibility.absences`, `nav_visibility(...)` Update, `is_active_for(...)` Update, `nav_items` Push
- [ ] `shifty-dioxus/src/api.rs` — 7 neue async-Funktionen
- [ ] `shifty-dioxus/src/loader.rs` — `load_absence_periods` + `load_vacation_balance` (+ `load_team_vacation`)
- [ ] `shifty-dioxus/src/error.rs` — ggf. `ShiftyError::Validation(String)` Variant für 422-Inline-Anzeige
- [ ] `shifty-dioxus/src/app.rs` — `use_coroutine(absence_service)` + `use_coroutine(vacation_balance_service)`
- [ ] `shifty-dioxus/Dioxus.toml` — `[[web.proxy]]` für `/absence-period` und `/vacation-balance`
- [ ] **Test infrastructure:** Unit-Tests für die Page-Module nutzen `dioxus-ssr` 0.6 Pattern (bereits etabliert in `dialog.rs:461`); kein neues Test-Framework nötig.

*Zusatz: i18n-Test-Pattern nach `i18n/mod.rs:425-457` — Test prüft, dass alle drei Locales Treffer für vorgegebenen Key-Set liefern.*

---

## Project Constraints (from CLAUDE.md)

**Top-level CLAUDE.md (`shifty/CLAUDE.md`):**
- Monorepo: `shifty-backend/` ist canonical, `shifty-dioxus/` ist Unterordner darin.
- Multi-language i18n: alle drei Locales (En/De/Cs) gleichzeitig pflegen.
- VCS: jj (NICHT `git commit`/`git add` aus Agent).
- Frontend WASM-Build-Gate: `cargo build --target wasm32-unknown-unknown` aus `shifty-dioxus/`.
- Tests: `cargo test` in beiden Workspaces grün als Pflicht.

**Backend (`shifty-backend/CLAUDE.md`):**
- Service-Tier-Konvention (Basic vs. BL): VacationBalanceService MUSS BL-Tier sein (D-04).
- `gen_service_impl!`-Macro für DI Pflicht.
- Transaction-Pattern: `Option<Self::Transaction>` + `transaction_dao.use_transaction(tx).await?` + `commit(tx)`.
- OpenAPI-Annotation `#[utoipa::path]` + `ToSchema` Derive Pflicht.
- DAO `WHERE deleted IS NULL` in jeder Read-Query.
- Snapshot Versioning: `CURRENT_SNAPSHOT_SCHEMA_VERSION` MUSS gebumpt werden bei `value_type`-Änderung — **Phase 8 ändert keine Snapshot-Felder, kein Bump nötig**.
- "Always execute cargo build, cargo test and cargo run (with some timeout) when you implement new features."

**User-private (`CLAUDE.local.md`):**
- NixOS: `nix develop` (NICHT `nix-shell`, shell.nix kaputt).
- jj-only-VCS: keine git-Calls, GSD-Auto-Commit ist `commit_docs: false`.

**Frontend (`shifty-backend/shifty-dioxus/CLAUDE.md`):**
- Component-Service-State-Pattern.
- Locale::En-statt-Locale::De-Bug-Avoidance.
- Tailwind-Watch-Mode während Dev (`npx tailwindcss -i ./input.css -o ./assets/tailwind.css --watch`).

**User-global (`/home/neosam/.claude/CLAUDE.md`):**
- "Always make sure you have tests for the changes" — Wave 0 Test-Files SIND erforderlich, nicht optional.

---

## Open Questions (RESOLVED)

> Alle fünf Fragen wurden im Planning-Schritt entschieden und sind durch die existierenden 6 Plans (08-01 bis 08-06) implementiert. RESOLVED-Suffix-Marker pro Frage als formales Closure-Signal.

1. **VacationBalance: ReportingService nutzen oder direkt aus AbsenceService aggregieren?**
   - What we know: ReportingService liefert `vacation_days`, `vacation_carryover`, `vacation_entitlement` (`reporting.rs:180-182`); ist aber teuer (lädt komplettes Employee-Report).
   - What's unclear: Plan-Phase muss entscheiden, ob VacationBalanceService ein dünner Wrapper über ReportingService wird (DRY, aber Performance) oder eine fokussierte Aggregation direkt (Performance, aber Logik-Duplikation).
   - Recommendation: Plan-Phase entscheidet basierend auf Performance-Anforderungen (HR-Liste mit N Personen → N ReportingService-Calls?).
   - **RESOLVED:** VacationBalanceService aggregiert direkt aus AbsenceService + EmployeeWorkDetailsService + CarryoverService (Plan 02 Task 1). KEINE ReportingService-Abhängigkeit (Performance: HR-Liste mit N Personen würde N teure ReportingService-Calls erzeugen; DRY-Vermeidung wird gegen Performance getauscht). Implementiert in `service_impl/src/vacation_balance.rs` via `gen_service_impl!`-BL-Tier mit Cross-Deps: `absence_service.find_by_sales_person`, `employee_work_details_service.find_by_sales_person_id` + `vacation_days_for_year(year)`, `carryover_service.get_carryover`.

2. **VacationBalance Cache-Strategy: per Page-Load fetchen oder GlobalSignal mit TTL?**
   - What we know: `ABSENCE_REFRESH`-Bump triggert ReList; analog `VACATION_BALANCE_REFRESH` möglich.
   - What's unclear: Reicht "bei jedem ABSENCE_REFRESH-Bump auch VacationBalance refetchen" oder soll TTL-basiert?
   - Recommendation: Erstere Variante (UI-SPEC § "Refresh-Flow" sagt explizit "VacationBalance wird ebenfalls bei jedem Refresh neu geladen").
   - **RESOLVED:** VacationBalance hat KEINEN dedizierten Cache und KEIN TTL. Frontend reuses `ABSENCE_REFRESH`-Token (Plan 04 Task 2 + Plan 05 Task 2). Jeder Create/Update/Delete bumpt `ABSENCE_REFRESH`; Page-`use_effect` reagiert auf Token-Änderung und re-fetched sowohl AbsenceList als auch VacationBalance. Konsequenz: VacationBalance ist immer konsistent zur Liste, KEIN stale state. Implementiert via `use_effect(move || { let _ = refresh_token; ... })` in `AbsencesPage`.

3. **`ShiftyError::Validation(String)` Variant hinzufügen oder über `Conflict` multiplex?**
   - What we know: Aktuell `ShiftyError::Conflict(msg)` ist 409-spezifisch.
   - What's unclear: 422 Self-Overlap unterscheidet sich semantisch (Bad-Banner vs. Warn-Banner per UI-SPEC). Plan-Phase MUSS Variant-Strategie wählen.
   - Recommendation: Neuer Variant `ShiftyError::Validation(String)` — semantisch sauber, 1 Zeile in `error.rs`.
   - **RESOLVED:** Neuer Variant `ShiftyError::Validation(String)` wurde in Plan 04 Task 1 hinzugefügt (`shifty-dioxus/src/error.rs`). Semantisch klar getrennt: 422 = Validation (bad-soft Banner für SelfOverlap, D-11), 409 = Conflict (warn-soft Banner für Version-Conflict, D-08). Konsumiert in `api::create_absence_period` + `api::update_absence_period` und in `AbsenceModal`-Submit-Side-Channel (Plan 05 Task 2 Sub-Step 1).

4. **`current sales_person_id` für Employee-Sicht — woher?**
   - What we know: `api::get_current_sales_person` existiert (`api.rs:261`); liefert `Option<SalesPersonTO>`.
   - What's unclear: Page muss Employee → eigene `sales_person_id` resolven; ist die schon im AUTH-State oder muss extra geladen werden?
   - Recommendation: Plan-Phase: Page-Mount-Effect ruft `api::get_current_sales_person` für Employee-Variant.
   - **RESOLVED:** Page lädt `current_sales_person_id` lazy via `api::get_current_sales_person()` beim Page-Mount (Plan 05 Task 2 Sub-Step 5). KEIN Pre-Load im AUTH-State, weil nicht alle Pages diesen Wert brauchen — Lazy-per-Page-Strategie ist konsistent mit existierendem Pattern (z. B. `page/employee_details.rs`). Speichert in `use_signal::<Option<Uuid>>` und triggert die Employee-Variant-Loader bei Resolve.

5. **OpenAPI-Snapshot-Pinning:** Nach v1.0 Phase 4 ist `insta`-Snapshot der OpenAPI-Surface aktiv. Plan-Phase muss bei Phase 8 Snapshot-Refresh dokumentieren (`cargo insta accept` oder ähnlich).
   - Recommendation: Plan-Phase Wave-1 (Backend) Final-Step: `cargo insta accept` ausführen + Snapshot ins Commit aufnehmen.
   - **RESOLVED:** Plan 03 ist dedicated für OpenAPI-Snapshot-Refresh. Workflow: (1) Plan 03 Task 1 lokalisiert den existierenden Snapshot-Test in `rest/`, (2) führt `cargo insta accept --workspace` aus, (3) verifiziert 3-Run-Determinism, (4) Plan 03 Task 2 ist `checkpoint:human-verify` mit Diff-Review-Step (W-6 Eskalation falls KEIN Snapshot-Test existiert). NEU per W-6 (revision): Plan 03 hat KEIN silent-pass mehr; falls kein Snapshot-Test in `rest/tests/` existiert, eskaliert Task 1 mit Wave-0-Stub-Pflicht.

---

## Sources

### Primary (HIGH confidence)

- **`shifty-backend/.planning/phases/08-absence-crud-page-foundation/08-CONTEXT.md`** — User-Decisions D-01..D-14, vollständig befolgt.
- **`shifty-backend/.planning/phases/08-absence-crud-page-foundation/08-UI-SPEC.md`** — Component-Inventory, Copywriting-Tabelle, State-Pattern.
- **`shifty-backend/.planning/REQUIREMENTS.md`** — FUI-A-01..04 (Phase 8 Scope).
- **`shifty-backend/.planning/STATE.md`** — Constraints In Force (VCS=jj, NixOS, i18n, rest-types-Cross-Crate, Service-Tier-Konvention).
- **`shifty-backend/.planning/ROADMAP.md`** — Phase 8 Goal + Success Criteria.
- **`shifty-backend/CLAUDE.md`** — Service-Tier-Konvention, gen_service_impl!, Transaction-Pattern, OpenAPI-Annotation.
- **`shifty-backend/shifty-dioxus/CLAUDE.md`** — Component-Service-State-Pattern, i18n.
- **`shifty-backend/CLAUDE.local.md`** — jj-only-VCS, NixOS `nix develop`.
- **Backend code (read fully):** `rest/src/absence.rs` (260 LOC, alle 6 Endpoints), `service/src/absence.rs` (295 LOC, Trait + Domain), `service_impl/src/absence.rs:1-120` (DI-Macro), `rest-types/src/lib.rs:1565..2040` (DTOs), `rest/src/lib.rs:120-396` (RestStateDef + error_handler), `service/src/reporting.rs:85-235` (EmployeeReport), `service_impl/src/reporting.rs:560-635` (vacation_entitlement-Berechnung), `shifty_bin/src/main.rs:194-355,683-815` (DI-Construction-Order).
- **Frontend code (read fully):** `src/router.rs` (51 LOC), `src/component/dialog.rs` (688 LOC), `src/component/form/inputs.rs` (390 LOC), `src/component/form/field.rs` (207 LOC), `src/component/atoms/btn.rs` (285 LOC), `src/component/extra_hours_modal.rs` (597 LOC), `src/component/top_bar.rs:1-200,300-400`, `src/api.rs:1-120,390-470,755-779`, `src/loader.rs:1-130`, `src/state/auth_info.rs` (28 LOC), `src/auth.rs` (25 LOC), `src/service/auth.rs` (55 LOC), `src/service/employee.rs` (361 LOC), `src/service/mod.rs` (20 LOC), `src/i18n/mod.rs` (552 LOC), `src/i18n/i18n.rs` (95 LOC), `src/i18n/de.rs:1-80`, `src/page/employee_details.rs:1-130`, `src/app.rs` (65 LOC), `src/error.rs` (44 LOC), `Cargo.toml` (82 LOC), `Dioxus.toml` (93 LOC).
- **Backend Brief:** `shifty-backend/shifty-dioxus/shifty-design/project/uploads/absence-feature-frontend.md` (vollständig gelesen, 4.6 Statuscodes-Tabelle bestätigt 200/201/204/403/404/409/422).
- **Notes/Seeds:** `.planning/notes/abwesenheiten-frontend-context.md`, `.planning/seeds/abwesenheiten-frontend-milestone.md`.
- **Backend Tests Pattern:** `service_impl/src/test/absence.rs` (1052 LOC, 30+ Tests via grep).

### Secondary (MEDIUM confidence)

- **Mockup als visual reference (not direct port):** `shifty-backend/shifty-dioxus/shifty-design/project/absences.jsx` (728 LOC) — UI-SPEC ist die kanonische Vorgabe; Mockup nur für Layout-Inspiration.
- **Service-Tier-Vergleichs-Cases:** `service_impl/src/carryover_rebuild.rs` (BL→ReportingService), `service_impl/src/booking_information.rs` (BL→6 Services). Pattern-Bestätigung für VacationBalance.

### Tertiary (LOW confidence)

- **Geschätzte i18n-Key-Anzahl** (~50): zählt UI-SPEC-Tabellen-Zeilen, exakte Anzahl ergibt sich beim Implementieren. `[ASSUMED]` A6.
- **Geschätzte Page-Datei-LOC** (800-1200): Extrapolation aus existierenden Pages. `[ASSUMED]` A7.

---

## Metadata

**Confidence breakdown:**

- Standard stack: **HIGH** — alle Crates bereits in Cargo.toml, alle Versionen verifiziert via direktes Lesen.
- Architecture (Frontend Page-Service-State + Backend BL-Tier-Service): **HIGH** — beide Patterns sind in der Codebase mehrfach präzedenziert (`employee.rs` + `absence.rs`).
- Resturlaubs-Endpoint-Shape: **MEDIUM** — DTO-Felder sind extrapoliert aus UI-SPEC i18n-Keys; Plan-Phase-Decision per D-04.
- DTO/Endpoint Inventarisierung: **HIGH** — alle DTOs gelesen, alle Endpoints gegen Source verifiziert.
- Pitfalls: **HIGH** — alle Pitfalls direkt aus Codebase-Patterns oder MEMORY.md/STATE.md `Constraints In Force` abgeleitet.
- Validation Architecture: **HIGH** — Test-Pattern in `service_impl/src/test/absence.rs` ist 1:1 anwendbar; Frontend dioxus-ssr 0.6 ist etabliert.

**Research date:** 2026-05-08
**Valid until:** 2026-06-07 (30 Tage; aktive Codebase, schnelle Iteration in v1.3-Milestone) — bei Wave-Start nochmals einen Quick-Re-Read der UI-SPEC fahren falls jene noch evolutioniert wird.
