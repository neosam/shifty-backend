---
title: Abwesenheiten-Frontend als v1.3-Milestone
trigger_condition: v1.2 (Frontend rest-types Konsolidierung) abgeschlossen
planted_date: 2026-05-07
---

# Abwesenheiten-Frontend als v1.3-Milestone

## Trigger

v1.2 (Phasen 6–7) ist abgeschlossen — `cargo build --target wasm32-unknown-unknown`
grün, Backend-`rest-types` als single source of truth verdrahtet, Match-Arme
exhaustiv, `dx serve` startet ohne Runtime-Panic.

Sobald das erfüllt ist: **`/gsd-new-milestone` für v1.3 fahren**, mit
"Frontend Abwesenheiten" als Kern-Scope.

## Skizze: Sub-Scopes (potenzielle Phasen)

Aus dem Frontend-Integrations-Brief
(`shifty-dioxus/shifty-design/project/uploads/absence-feature-frontend.md`)
und dem 729-Zeilen-Mockup
(`shifty-dioxus/shifty-design/project/absences.jsx`) lassen sich
ungefähr diese Phasen ableiten:

### Phase A — Absence-CRUD-Page (Kern)

- Neue Route `absences` + Page-Komponente in `src/page/`
- `api.rs` und `loader.rs` Erweiterung für `/absence-period` (GET-list,
  GET-by-id, POST, PUT, DELETE, GET by-sales-person)
- Service- und State-Module für Absence-Listen, Selektion, Form-State
- Form-Komponente: Datum-Range-Picker (Ganztage), Kategorie-Dropdown
  (Vacation / SickLeave / UnpaidLeave), Description-Feld
- HR-Sicht (`hr`-Privileg): Filter über alle Mitarbeiter
- Employee-Sicht (kein `hr`-Privileg): nur eigene Liste
- Warnings-Anzeige aus `AbsencePeriodCreateResultTO.warnings[]`
- i18n De / En / Cs (Page-Titel, Kategorie-Labels, Form-Labels,
  Warning-Texte)

### Phase B — Booking-Flow Umstellung + Reverse-Warnings

- `POST /shiftplan-edit/booking` statt `POST /booking` für die
  Buchungs-Aktionen im Shiftplan-Editor
- `BookingCreateResultTO.warnings[]` als nicht-blockierender
  Confirm-Dialog ("Buchung auf Urlaubstag von X. Trotzdem anlegen?")
- `POST /shiftplan-edit/copy-week` für Wochen-Kopie inkl. Warning-Aggregat
- Alter `POST /booking` bleibt für Bestands-Calls (kein Hard-Cutover im
  Frontend nötig)

### Phase C — Shiftplan-View mit `UnavailabilityMarker`

- Umstellung der Shiftplan-Wochen-View auf den per-sales-person Endpoint
- Visuelle Marker pro Tag pro Person:
  - `absence_period` (mit Kategorie-Farbe)
  - `manual_unavailable` (klassischer manueller Eintrag)
  - `both` — eigene Visual-Indication: signalisiert User, dass der
    manuelle Unavailable-Eintrag nach Cutover redundant geworden ist
- Möglicher Aufräum-Button "manuellen Eintrag löschen" für `both`-Tage

### Phase D — Migrations-Hinweis-UX

- Bestehende "Urlaub eintragen"-Buttons via `extra_hours` (in
  `add_extra_hours_form.rs`, `extra_hours_modal.rs`,
  `add_extra_days_form.rs`, `add_extra_hours_choice.rs`) prüfen
- Vor Cutover: auf neue Maske umlinken (Soft-Migration)
- Nach Cutover: `403 ExtraHoursCategoryDeprecatedErrorTO` abfangen,
  Toast/Banner mit Migrations-Hinweis und Link zur Abwesenheiten-Maske

### (Optional) Phase E — Admin-Cutover-Maske

- Nur falls die Migration via UI angestoßen werden soll (statt CLI)
- `POST /admin/cutover/{gate-dry-run, commit, profile}`
- Strenge Permission-Gating: `cutover_admin` für `commit`

## Reihenfolge

A → B parallel zu C → D. E ist optional und kann auch außerhalb des
Milestones bleiben.

## Risiken / offene Fragen

- **i18n-Volumen**: Drei Locales × viele neue Strings — größerer Block
  als sonst, sollte in Phase A mit eingeplant werden, nicht nachgereicht.
- **Cutover-Status zur Implementation-Zeit**: Falls der Backend-Flag
  `absence_range_source_active` zwischen Frontend-Phasen-Start und
  -Ende geflippt wird, sind unterschiedliche Code-Pfade im Frontend
  aktiv. Strategie: Frontend defensiv beide Pfade unterstützen (lesen
  immer aus `/absence-period`; Schreiben über alte `extra_hours`-Maske
  nur falls Flag noch aus, sonst über neue Maske).
- **Mockup-Tweak `viewAs`**: Im Mockup ein UI-Toggle, im echten
  Frontend muss es aus dem Auth-Context kommen (`hr`-Privileg-Check) —
  nicht aus einem User-Preference.
- **Warning-UX**: Confirm-Dialog im Mockup ist
  `window.confirm` — im echten Frontend braucht das einen schöneren
  Dialog (Dioxus Dialog-Komponente), siehe `component/dialog.rs`.

## Verweise

- Begleitnote: [`abwesenheiten-frontend-context`](../notes/abwesenheiten-frontend-context.md)
- Brief: `shifty-dioxus/shifty-design/project/uploads/absence-feature-frontend.md`
- Mockup: `shifty-dioxus/shifty-design/project/absences.jsx`
