---
title: Abwesenheiten-Frontend — Kontext und Quellen
date: 2026-05-07
context: explore session — Frontend-Sicht für die in v1.0 geshippte range-based Absence-Domain
---

# Abwesenheiten-Frontend — Kontext und Quellen

## Kurzzusammenfassung

Die **Backend-Domäne** für range-based Abwesenheiten (`AbsencePeriod`) ist
in v1.0 vollständig geshipped (Phasen 1–4, 433+ Tests grün, 2026-05-03).
Endpunkte, DTOs, Reporting-Integration, Booking-Konflikt-Warnings und
Cutover-Mechanik stehen.

Das **Frontend** hat aktuell keine Maske dafür. Der einzige bestehende
Eingangsweg für `Vacation` / `SickLeave` / `UnpaidLeave` ist das alte
single-day `extra_hours`-Schema, das nach Cutover-Flag-Flip mit
`403 ExtraHoursCategoryDeprecatedErrorTO` antwortet.

Diese Lücke schließen wir mit einer neuen Top-Level-Maske
**"Abwesenheiten"** in der Dioxus-App.

## Quellen im Repo

- **Frontend-Integrations-Brief**:
  `shifty-dioxus/shifty-design/project/uploads/absence-feature-frontend.md`
  — Backend-Stand, API, DTOs, 6 empfohlene Frontend-Maßnahmen
- **Mockup (729 Zeilen JSX)**:
  `shifty-dioxus/shifty-design/project/absences.jsx` — `AbsencePage`
  Komponente mit HR- und Employee-Sicht
- **Preview-Einbettung mit Tweak**:
  `shifty-dioxus/shifty-design/project/Shifty Preview.html` — Route
  `absences`, Tweak `viewAs: "hr" | "employee"` schaltet zwischen
  beiden Sichten, Menü-Restriction für Employee-Rolle
- **Backend-DTOs**: `rest-types/src/lib.rs:1565..2040`
- **Backend-Endpoints**: `rest/src/absence.rs`, `rest/src/shiftplan_edit.rs`,
  `rest/src/cutover.rs`

## Sechs Frontend-Maßnahmen aus dem Brief

1. **Neue Maske "Abwesenheits-Zeiträume"** mit CRUD gegen
   `/absence-period`. POST/PUT-Antwort enthält
   `AbsencePeriodCreateResultTO.warnings[]` — als nicht-blockierende
   Hinweisliste rendern.
2. **Booking-Flow umstellen** auf `POST /shiftplan-edit/booking` (statt
   `POST /booking`), damit Reverse-Warnings ("Buchung auf Urlaubstag")
   sichtbar werden. Alter Endpoint bleibt parallel bestehen.
3. **Shiftplan-Wochen-View** auf den per-sales-person Endpoint
   `/shiftplan-info/{shiftplan_id}/{year}/{week}/sales-person/{sales_person_id}`
   umstellen, um `unavailable: Option<UnavailabilityMarkerTO>` farbig
   darzustellen. Variante `both` braucht eigene Visual-Indication
   (signalisiert redundanten manuellen Unavailable-Eintrag nach Cutover).
4. **Migrations-Hinweis-UX**: Alte "Urlaub eintragen"-Buttons via
   `extra_hours` entfernen oder zumindest auf
   `403 ExtraHoursCategoryDeprecatedErrorTO` reagieren und auf neue
   Maske umleiten.
5. **i18n**: Neue Strings in De / En / Cs (Kategorie-Labels,
   Warning-Texte, Deprecation-Hinweis).
6. **OpenAPI**: Backend liefert komplette Schemas via Swagger UI (Phase 4
   hat OpenAPI-Snapshot-Pinning per `insta` aktiviert).

## Architekturnotiz: HR ∨ self

Backend-Permission ist bereits `hr ∨ self` — d. h. eingeloggter User
kann eigene Absences anlegen, HR-Privileg-Träger können alle anlegen.
Das Mockup nutzt Tweak `viewAs` zur Demo; **im echten Frontend** kommt
die Sicht aus dem Auth-Context, nicht aus einem Toggle. Das Menü zeigt
für Employees ein reduziertes Set: `[shiftplan, my-shifts, overview,
absences]`.

## Reihenfolge gegenüber laufendem v1.2

v1.2 (rest-types Frontend-Konsolidierung) ist Voraussetzung — die neuen
DTOs `AbsencePeriodTO`, `WarningTO`, `UnavailabilityMarkerTO`,
`UnavailabilityMarkerTO.kind` müssen sauber im Frontend referenzierbar
sein, ohne lokale Mapping-Workarounds. Die Match-Arm-Erschöpftheit
(FC-01) und die Compile-Gate (FC-02) räumen den Weg dafür frei.

**Reihenfolge**: v1.2 fertig → v1.3 startet mit Abwesenheiten-Frontend.

## Cutover-Status

Der Backend-Feature-Flag heißt `absence_range_source_active`. Vor dem
Flip schreiben Frontends weiter auf `extra_hours`; nach dem Flip ist
`POST /extra-hours` für die drei Kategorien deprecated. Der Flag-Status
muss in der Frontend-Phase berücksichtigt werden (UX, Error-Handling).

## Out of Scope der Frontend-Phase

- Halbtage / Stundenebene (Backend modelliert nur Ganztage)
- Genehmigungs-Workflow (Backend kennt keinen Approval-Schritt; Anlage
  ist atomar mit `hr ∨ self`-Permission)
- Admin-Cutover-Maske (`/admin/cutover/*` ist getrenntes Admin-Surface,
  optional separat)
