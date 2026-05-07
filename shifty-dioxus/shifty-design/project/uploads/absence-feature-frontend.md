# Range-Based Absence Management — Frontend-Integrations-Brief

**Backend-Milestone:** v1.0 (geshipped 2026-05-03, Phasen 1–4)
**Stand:** Phasen 1–4 vollständig durch, 23 Pläne, 433+ Tests grün workspace-weit.

---

## 1. Zweck

Bisher wurden Vacation / SickLeave / UnpaidLeave als **einzelne Tageseinträge mit Stundenbetrag** (`extra_hours`) erfasst — was bei Vertragsänderungen zu manueller Nacharbeit führte und mit dem Shiftplan in Doppel-Eintragung kollidierte (`extra_hours` + `sales_person_unavailable`).

Die neue **Absence-Domain** modelliert Abwesenheiten als **Zeiträume** (`from_date` / `to_date`). Die Stunden pro Tag werden zur Reporting-Zeit aus dem **am jeweiligen Tag gültigen Arbeitsvertrag** abgeleitet (`derive_hours_for_range`). Damit überleben Abwesenheiten Vertragsänderungen, Feiertage werden orthogonal mit 0 h verrechnet, und Doppel-Eintragung ist abgeschafft.

---

## 2. Fachliche Änderungen

| Punkt              | Verhalten |
|--------------------|-----------|
| Granularität       | **Nur Ganztage**. Halbtage / Stundenebene sind in v1 nicht modelliert. |
| Kategorien         | `Vacation`, `SickLeave`, `UnpaidLeave` (drei Werte) |
| Vertragsänderung   | **Prospektiv**: alte Perioden bleiben unverändert, neue Perioden rechnen mit dem zum Zeitpunkt gültigen Vertrag |
| Feiertage          | Auf einem Feiertag = **0 h Urlaub** (separate Feiertags-Gutschrift bleibt unberührt) |
| Booking-Konflikt   | **Warnung, nicht-blockierend**. Buchung/Anlage wird trotzdem persistiert, kein Auto-Cleanup, kein Auto-Löschen |
| Self-Overlap       | Eine Person darf sich nicht **selbst** mit derselben Kategorie überlappen (Server lehnt `422` ab) |
| Permission         | `hr`-Privileg ODER der eingeloggte User ist der `sales_person_id` der Absence ("HR ∨ self") |
| Soft-Delete        | Löschen setzt `deleted` (logical_id-versioniert) — Datensatz bleibt für Audit-Logs vorhanden |
| Cutover            | Feature-Flag `absence_range_source_active`. Vor dem Flip schreiben Frontends weiterhin auf `extra_hours`. Nach dem Flip ist `POST /extra-hours` für die drei Kategorien **deprecated → 403** |

---

## 3. Technische Änderungen (Backend)

- Neue Schicht `service::absence` + DAO + REST-Layer (Phase 1)
- Reporting integriert Absences via `derive_hours_for_range` (Phase 2). `CURRENT_SNAPSHOT_SCHEMA_VERSION` ist von 2 → 3 gebumpt
- Cross-Source-Warnings: Forward (Absence überlappt Booking / manuelle Unavailability) und Reverse (Booking auf Absence-Tag) — Phase 3
- Shiftplan-View kennt `UnavailabilityMarker` (AbsencePeriod / Manual / Both) per Tag pro Sales-Person — die bisherige Doppel-Eintragung entfällt
- Cutover-Service mit Heuristik-Migration aus Bestand-`extra_hours` + Drift-Gate (`< 0.01 h` pro `(sp, kategorie, jahr)`) + atomarer Commit-Tx (Migration + Carryover-Refresh + Soft-Delete + Flag-Flip)
- `extra_hours.update` neu real implementiert (war vorher `unimplemented!()`), mit logical_id-Versionierung

Frontend-relevant: Die existierenden `POST /extra-hours`-Endpunkte für `Vacation/SickLeave/UnpaidLeave` sind **flag-gated** — nach Flag-Flip antwortet der Server `403` mit `ExtraHoursCategoryDeprecatedErrorTO`.

---

## 4. API-Änderungen

### 4.1 Neu: `/absence-period` (Phase 1)

| Method | Path                                                     | Body              | Response                                | Permission |
|--------|----------------------------------------------------------|-------------------|-----------------------------------------|------------|
| POST   | `/absence-period`                                        | `AbsencePeriodTO` | `201 AbsencePeriodCreateResultTO`       | hr ∨ self  |
| GET    | `/absence-period`                                        | —                 | `200 [AbsencePeriodTO]`                 | hr ∨ self  |
| GET    | `/absence-period/{id}`                                   | —                 | `200 AbsencePeriodTO`                   | hr ∨ self  |
| PUT    | `/absence-period/{id}`                                   | `AbsencePeriodTO` | `200 AbsencePeriodCreateResultTO`       | hr ∨ self  |
| DELETE | `/absence-period/{id}`                                   | —                 | `204`                                   | hr ∨ self  |
| GET    | `/absence-period/by-sales-person/{sales_person_id}`      | —                 | `200 [AbsencePeriodTO]`                 | hr ∨ self  |

**Hinweis zu PUT:** `path-id wins` — der Body-`id` wird vom Server überschrieben. Frontend kann sich auf den URL-Pfad verlassen.

### 4.2 Geändert: konflikt-aware Booking (Phase 3)

**Neu (zusätzlich zum bestehenden `POST /booking`):**

| Method | Path                          | Body                                                               | Response                              |
|--------|-------------------------------|--------------------------------------------------------------------|---------------------------------------|
| POST   | `/shiftplan-edit/booking`     | `BookingTO`                                                        | `201 BookingCreateResultTO`           |
| POST   | `/shiftplan-edit/copy-week`   | `CopyWeekRequest` (`from_year`, `from_calendar_week`, `to_year`, `to_calendar_week`) | `200 CopyWeekResultTO` |

`POST /booking` (alter Endpoint) bleibt **unverändert** und liefert kein Warning-Wrapper — Regression-Lock D-Phase3-18. Für Konflikt-Anzeige im Frontend → die neuen `/shiftplan-edit/*`-Endpunkte verwenden.

### 4.3 Geändert: Shiftplan-View per Sales-Person (Phase 3)

| Method | Path                                                                                  | Response                                                              |
|--------|---------------------------------------------------------------------------------------|-----------------------------------------------------------------------|
| GET    | `/shiftplan-info/{shiftplan_id}/{year}/{week}/sales-person/{sales_person_id}`         | week-View, jeder `ShiftplanDayTO.unavailable: Option<UnavailabilityMarkerTO>` |
| GET    | `/shiftplan-info/day/{year}/{week}/{day_of_week}/sales-person/{sales_person_id}`      | day-View analog                                                       |

`UnavailabilityMarkerTO` ist ein Tag-Enum mit `kind = absence_period | manual_unavailable | both`. Die Variante `Both` wird gesetzt, wenn an einem Tag sowohl eine `AbsencePeriod` als auch ein manueller `sales_person_unavailable`-Eintrag existiert (D-Phase3-16: kein Auto-Cleanup).

### 4.4 Neu: `/admin/cutover` (Phase 4 — Migrations-Admin-Surface)

| Method | Path                          | Body | Response                                         | Permission       |
|--------|-------------------------------|------|--------------------------------------------------|------------------|
| POST   | `/admin/cutover/gate-dry-run` | `{}` | `200 CutoverRunResultTO` (Tx wird zurückgerollt) | hr               |
| POST   | `/admin/cutover/commit`       | `{}` | `200 CutoverRunResultTO` (atomare Tx)            | `cutover_admin`  |
| POST   | `/admin/cutover/profile`      | `{}` | `200 CutoverProfileTO` (+ schreibt JSON-File)    | hr               |

Reine Admin-Surface — relevant für Frontend nur, falls eine Admin-Maske die Migration anstoßen soll. Sonst ignorieren.

### 4.5 Neue / geänderte DTOs (alle in `rest-types`)

```ts
// Hauptentität
AbsencePeriodTO {
  id: Uuid                          // 0-Uuid bei POST → Server vergibt
  sales_person_id: Uuid
  category: "Vacation" | "SickLeave" | "UnpaidLeave"
  from_date: "YYYY-MM-DD"
  to_date:   "YYYY-MM-DD"           // inklusiv
  description: string
  created: ISO-Datetime | null
  deleted: ISO-Datetime | null
  $version: Uuid                    // optimistic locking
}

// Wrapper für POST/PUT /absence-period
AbsencePeriodCreateResultTO {
  absence: AbsencePeriodTO
  warnings: WarningTO[]             // Forward-Warnings (Booking/Manual überlappt)
}

// Wrapper für POST /shiftplan-edit/booking
BookingCreateResultTO {
  booking: BookingTO
  warnings: WarningTO[]             // Reverse-Warnings (Booking auf Absence/Unavailable)
}

// Wrapper für POST /shiftplan-edit/copy-week
CopyWeekResultTO {
  copied_bookings: BookingTO[]
  warnings: WarningTO[]             // KEINE De-Dup zwischen Quellen
}

// Tag-Enum: { "kind": "<snake_case>", "data": { ... } }
WarningTO =
  | { kind: "booking_on_absence_day",              data: { booking_id, date, absence_id, category } }
  | { kind: "booking_on_unavailable_day",          data: { booking_id, year, week, day_of_week } }
  | { kind: "absence_overlaps_booking",            data: { absence_id, booking_id, date } }
  | { kind: "absence_overlaps_manual_unavailable", data: { absence_id, unavailable_id } }
  | { kind: "paid_employee_limit_exceeded",        data: { slot_id, booking_id, year, week, current_paid_count, max_paid_employees } }   // Phase 5

// Per-Tag-Marker im per-sales-person Shiftplan-View
UnavailabilityMarkerTO =
  | { kind: "absence_period",      data: { absence_id, category } }
  | { kind: "manual_unavailable" }
  | { kind: "both",                data: { absence_id, category } }

// Deprecation-Body bei POST /extra-hours mit gesperrter Kategorie
ExtraHoursCategoryDeprecatedErrorTO {
  error: "extra_hours_category_deprecated"
  category: "vacation" | "sickleave" | "unpaidleave"
  message: string                   // user-facing Migrations-Hinweis
}
```

### 4.6 Statuscodes

- `201` POST erfolgreich
- `200` GET / PUT / Cutover
- `204` DELETE
- `403` Forbidden (Permission ODER deprecated category nach Flag-Flip)
- `404` Not Found / soft-deleted
- `409` Version-Konflikt (PUT mit veraltetem `$version`)
- `422` Validation (Self-Overlap, ungültiger Range, Schema-Constraint)

---

## 5. Empfohlene Frontend-Maßnahmen

1. **Neue Maske "Abwesenheits-Zeiträume"** mit CRUD gegen `/absence-period`. Nach POST/PUT die `warnings[]` aus `AbsencePeriodCreateResultTO` als nicht-blockierende Hinweisliste rendern (z. B. "Achtung: existierendes Booking am 12.07.2026 überlappt diese Abwesenheit").
2. **Booking-Flow umstellen** auf `POST /shiftplan-edit/booking` (statt `POST /booking`), damit Reverse-Warnings ("Du buchst auf einem Urlaubstag von Person X") sichtbar werden. Alter Endpoint bleibt für Bestands-Calls bestehen.
3. **Shiftplan-Wochen-View** auf den per-sales-person Endpoint umstellen, um `unavailable`-Marker (mit `category` aus der Absence) farbig darzustellen. Variante `both` braucht eigene Visual-Indication, da sie dem User signalisiert, dass eine `ManualUnavailable` nach Cutover redundant geworden ist.
4. **Migrations-Hinweis-UX**: Falls in der UI noch Buttons "Urlaub eintragen" via `extra_hours` existieren — diese vor dem Cutover entfernen oder zumindest auf den `403 ExtraHoursCategoryDeprecatedErrorTO` reagieren und den User auf die neue Maske umleiten.
5. **i18n**: Neue Strings in De / En / Cs anlegen (Kategorie-Labels, Warning-Texte, Deprecation-Hinweis).
6. **OpenAPI**: Backend liefert komplette Schemas via Swagger UI — falls ihr aus OpenAPI Client-Code generiert, neuen Snapshot ziehen (Phase 4 hat OpenAPI-Snapshot-Pinning per `insta` aktiviert, REST-Surface ist stabil).

---

## Quellen im Repo

- `.planning/milestones/v1.0-ROADMAP.md` — Milestone-Übersicht
- `.planning/phases/0{1..4}-*/0{N}-VERIFICATION.md` — D-Decisions je Phase
- `rest/src/absence.rs` / `rest/src/shiftplan_edit.rs` / `rest/src/cutover.rs` — Endpoint-Quellen
- `rest-types/src/lib.rs:1565..2040` — DTO-Definitionen
