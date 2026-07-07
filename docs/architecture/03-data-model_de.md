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
- **Rebooking** (v2.6, Phase 54) — audit-fähige Batches, die
  Freiwillig-Stunden gedeckelter Mitarbeiter*innen ausgleichen; siehe
  Feature [F14](../features/F14-rebooking.md).
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
| 2026-07 | v2.6 Phase 54 — Tabellen `rebooking_batch` + `rebooking_batch_entry`, Marker-Spalte `extra_hours.source`, Voluntary-Rebooking-Toggle-Seed. |

Die vollständige Liste steht in `migrations/sqlite/` — Dateinamen sind
sortiert nach Zeitstempel-Prefix.

## Wo das Datenmodell "spricht"

- **Basic-Service-DAOs** kennen ihre Tabelle direkt.
- **Business-Logic-Services** kennen keine DAOs außer die transitiv
  über konsumierte Basic-Services.
- **Reporting** liest über mehrere Aggregate hinweg und ist damit die
  Stelle, an der Schema-Änderungen am schnellsten Kontakt zur
  Balance-Rechnung bekommen.

## Phase 54 (v2.6) — Rebooking-Datenmodell-Ergänzungen

Migration-Set `20260707000000..02` legt das Datenmodell-Fundament
für Feature [F14](../features/F14-rebooking.md).

### `rebooking_batch` (Parent)

Audit-fähiger Batch gepaarter `extra_hours`-Zeilen, die
Freiwillig-Stunden eines/einer gedeckelten Mitarbeiter*in in einer
ISO-Woche ausgleichen.

| Spalte | Typ | Anmerkung |
| --- | --- | --- |
| `id` | BLOB(16) PK | UUID v4. |
| `sales_person_id` | BLOB(16) | Mitarbeiter, auf den der Batch bucht. |
| `iso_year`, `iso_week` | INT | ISO-Jahr + ISO-Woche des Reconciliation-Fensters. |
| `kind` | TEXT | `Manual` \| `HrSuggestion` \| `AutoCron` \| `AutoCronBackfill`. |
| `state` | TEXT | `Pending` \| `Approved` \| `Rejected` \| `SkippedLocked`. |
| `created`, `approved`, `approved_by` | TEXT | Audit-Zeitstempel + Username; `approved*` NULL bis `state = Approved`. |
| `deleted` | TEXT nullable | Soft-Delete-Marker. |
| `update_process`, `update_version` | Audit-Spalten |

**Constraint [D-54-DM-01]** — partieller UNIQUE-Index
`rebooking_batch_week_unique_idx` auf
`(sales_person_id, iso_year, iso_week) WHERE deleted IS NULL` —
*global über alle `kind`s*. Claim-on-Suggest: sobald HR einen
Pending-Batch für Woche X öffnet, kann der Phase-56-F4-Cron nicht
mit einem zweiten `AutoCron`-Batch für dieselbe Woche reinlaufen.
Zwei Performance-Indices existieren daneben
(`rebooking_batch_state_idx` auf `state`,
`rebooking_batch_entry_sp_idx` auf `sales_person_id` in der
Child-Tabelle).

### `rebooking_batch_entry` (Child)

Slot-Payload pro Batch, eine Zeile pro umgebuchtem Stundenpaket.

| Spalte | Typ | Anmerkung |
| --- | --- | --- |
| `id` | BLOB(16) PK | |
| `batch_id` | BLOB(16) FK → `rebooking_batch(id)` | Kein CASCADE — Soft-Delete-Muster. |
| `sales_person_id` | BLOB(16) | Denormalisiert für Query-Performance. |
| `hours` | REAL | Absolute Stundenzahl, die umgebucht wird. |
| `balance_before` | REAL | Balance-Snapshot zum Vorschlagszeitpunkt (Audit). |
| `voluntary_actual`, `voluntary_committed` | REAL | Snapshot von F1-Zähler + F2-pro-rata-Soll zum Vorschlagszeitpunkt. |
| `extra_hours_out_id`, `extra_hours_in_id` | BLOB(16) nullable | FKs in `extra_hours` — atomar gesetzt beim Übergang state → Approved (Phase-55-Writer). |
| `created`, `deleted`, `update_process`, `update_version` | Audit-Spalten |

### `extra_hours.source` Marker-Spalte ([D-54-DM-02])

Additive Spalte `source TEXT NOT NULL DEFAULT 'manual'`. Zwei aktive
Domain-Werte:

- **`manual`** — jede Zeile aus HR-CRUD, Absence-Convert, Dev-Seed und
  REST-TO-Writern. Bestandszeilen migrieren per Column-DEFAULT.
- **`rebooking`** — reserviert für F3/F4/F5-Writer (Phase 55+). In
  Phase 54 setzt kein Writer diesen Wert.

Reader-Konsequenz: Aggregate, die unter zukünftigen Rebooking-Paaren
balance-neutral bleiben müssen, filtern `source = 'manual'`. Erster
Live-Konsument ist `voluntary_ist_total_for_year(..)` in
`service_impl/src/reporting.rs`.

### Toggle-Seed `voluntary_rebooking_auto_active_from`

Migration `20260707000002_seed-voluntary-rebooking-toggle.sql`
seedet idempotent `INSERT OR IGNORE INTO toggle` mit
`enabled = 0, value = NULL`. In Phase 54 inaktiv — der F4-Cron in
Phase 56 liest den ISO-Datums-Wert, um die AutoCron-Writer-Kette zu
aktivieren.

## Verwandte Randfälle

- Soft-Delete-Konsistenz → [`../domain/edge-cases.md#8-soft-delete-konsistenz`](../domain/edge-cases.md#8-soft-delete-konsistenz)
- `.sqlx`-Cache und Migrations → [`../domain/edge-cases.md#10-migrations--sqlx-offline-cache`](../domain/edge-cases.md#10-migrations--sqlx-offline-cache)
