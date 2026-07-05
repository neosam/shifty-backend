# Datenmodell

## Physisches Schema (ER)

Das aktuelle ER-Modell wird automatisch aus den `migrations/sqlite/`-
Dateien generiert und ist in
[`diagrams/db-schema-er.mmd`](./diagrams/db-schema-er.mmd) als
Mermaid-Diagramm hinterlegt.

## Logisches Modell (Aggregate)

Die Aggregat-Sicht — welche Objekte fachlich zusammengehören und wie
Business-Regeln die Grenzen ziehen — ist in
[`diagrams/domain-aggregates.mmd`](./diagrams/domain-aggregates.mmd).

Kern-Aggregate:

- **Employee** — Sales Person + Work Details (Contract) + Unavailable + Vacation Offset.
- **Shiftplan** — Slots + Catalog + Special Days.
- **Booking** — Booking + Booking Log.
- **Absence** (v1.0+) — range-basierte Abwesenheiten.
- **Legacy Extra Hours** (vor Cutover) — Single-Day-Zeilen + Custom-Kategorien.
- **Accounting** — Carryover + Vacation Balance + Report.
- **Billing Period** — Snapshot mit `snapshot_schema_version`.
- **Week Metadata** — Week Status + Week Message.
- **Auth & Session** — User + Role + Session + Invitation.
- **System** — Feature Flag + Toggle + Scheduler.

## Konventionen im Schema

### Soft-Delete

Fast jede Tabelle hat eine `deleted` (nullable timestamp) Spalte. Reader
filtern **immer** `WHERE deleted IS NULL`. Das ist eine Konvention, kein
DB-Constraint — d.h. Reviewer müssen prüfen, dass jeder neue
`query!`/`query_as!` diesen Filter setzt.

### Time-Spalten

- **`from` / `to` (Date):** Halb-offene Ranges? Geschlossene? Konvention
  variiert pro Tabelle. **[Zu prüfen]** in `absence`, `employee_work_details`,
  `sales_person`.
- **Timestamps:** Speichern SQLite-native. Zeitzone: **[Zu prüfen]**
  (UTC vs Local-Berlin).

### Foreign Keys

SQLite prüft FKs nur, wenn `PRAGMA foreign_keys = ON` aktiv ist. Ob
Shifty das global setzt: **[Zu prüfen]** im Startup-Pfad.

### Views

Manche Aggregat-Reads gehen über Views (`bookings_view` — Migration
`20240728155625_add-bookings-view.sql`, erweitert um User-Tracking in
`20250115000001_update-bookings-view-add-user-tracking.sql`). Views sind
komfortabel, aber ändern sich mit ihren zugrunde liegenden Tabellen —
Migration-Ordnung beachten.

### Enum-Spalten

Kategorien wie `ExtraHoursCategory` werden als `TEXT` mit Enum-Namen
persistiert (z.B. `"ExtraWork"`, `"Vacation"`). Neue Varianten brauchen
sowohl Code-Change (Enum-Erweiterung + Konvertierung) als auch
DB-Migration, wenn ein Constraint / Check die Werte einschränkt.

**[Zu prüfen]** — ob es explizite CHECK-Constraints auf Category-Spalten
gibt.

## Migration-Historie (High-Level)

Chronologisch geordnet — Details siehe die einzelnen Feature-Dokus:

| Zeitraum | Meilenstein |
| --- | --- |
| 2024-04 | User & Rollen, initiale RBAC. |
| 2024-05 – 2024-07 | Slot + Sales Person + Booking + Booking-View. |
| 2024-08 | `min_resources`-Spalte. |
| 2024-10 | Special-Days-Tabelle, Weekday + Vacation auf Working Days. |
| 2024-11 | Session-Tabelle, Constraint-Verschärfung. |
| 2024-12 | Yearly Carryover (Hours + Vacation). |
| 2025-01 | User-Tracking auf Bookings + View, Week-Message. |
| 2025-04 | Custom Extra Hours. |
| 2025-08 | Billing Period, Text Template. |
| 2025-10 | User Invitation, Session-Tracking. |
| 2026+ | Absence-System-Cutover, Multi-Plan-Support. |

Die vollständige Liste steht in `migrations/sqlite/` — Dateinamen sind
sortiert nach Zeitstempel-Prefix.

## Wo das Datenmodell "spricht"

- **Basic-Service-DAOs** kennen ihre Tabelle direkt.
- **Business-Logic-Services** kennen keine DAOs außer die transitiv
  über konsumierte Basic-Services.
- **Reporting** liest über mehrere Aggregate hinweg und ist damit die
  Stelle, an der Schema-Änderungen am schnellsten Kontakt zur
  Balance-Rechnung bekommen.

## Verwandte Randfälle

- Soft-Delete-Konsistenz → [`../domain/edge-cases.md#8-soft-delete-konsistenz`](../domain/edge-cases.md#8-soft-delete-konsistenz)
- `.sqlx`-Cache und Migrations → [`../domain/edge-cases.md#10-migrations--sqlx-offline-cache`](../domain/edge-cases.md#10-migrations--sqlx-offline-cache)
