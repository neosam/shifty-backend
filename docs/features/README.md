# Features — Ein Dokument pro Domäne

Diese Sektion ist die **Feature-Referenz**. Jede Datei beschreibt ein
Feature-Cluster von Shifty vollständig: fachlich, technisch, mit
Randfällen und Test-Coverage.

## Struktur einer Feature-Doku

Jedes Feature-Dokument folgt derselben Gliederung:

1. **Was ist das?** — Fachliche Kurzbeschreibung, Zielgruppe im UI.
2. **Fachliche Regeln** — Alle Business-Constraints ausformuliert.
3. **Datenmodell** — Tabellen, Spalten, Beziehungen, relevante Migrations.
4. **Service-API** — Trait-Methoden, Auth-Gates, TX-Verhalten.
5. **REST-Endpoints** — Pfade, Methoden, DTOs, Fehlerfälle.
6. **Frontend-Integration** — Welche Pages / Components das Feature nutzen.
7. **Randfälle** — Verweise auf `../domain/edge-cases.md` + feature-spezifische.
8. **Tests** — Wo Unit/Integration-Coverage liegt, was NICHT abgedeckt ist.
9. **Historie** — Milestone-Kontext, warum das Feature so aussieht (Cutover,
   Toggle-Rollout, etc.).

## Feature-Cluster

| # | Cluster | Datei |
| --- | --- | --- |
| F01 | Employee Management (Sales Person, Kontrakt, Unavailability) | [F01-employee-management.md](./F01-employee-management.md) |
| F02 | Shiftplan Core (Slots, Katalog, Editor, Ansicht) | [F02-shiftplan-core.md](./F02-shiftplan-core.md) |
| F03 | Booking (Zuweisung, Log, Information) | [F03-booking.md](./F03-booking.md) |
| F04 | Extra Hours — Legacy Zeit-Erfassung + Custom-Kategorien | [F04-extra-hours.md](./F04-extra-hours.md) |
| F05 | Absence System (Range-basiert, v1.0+) | [F05-absence-system.md](./F05-absence-system.md) |
| F06 | Vacation Management (Balance, Offset, Carryover) | [F06-vacation-management.md](./F06-vacation-management.md) |
| F07 | Reporting & Balance-Berechnung | [F07-reporting-balance.md](./F07-reporting-balance.md) |
| F08 | Billing Period (Snapshot + Versioning) | [F08-billing-period.md](./F08-billing-period.md) |
| F09 | Special Days, Week Status, Week Message, Warning | [F09-week-metadata.md](./F09-week-metadata.md) |
| F10 | Templates & Kommunikation (Text-Templates, User Invitation) | [F10-templates-communication.md](./F10-templates-communication.md) |
| F11 | Export (PDF-Shiftplan, iCal, WebDAV, Scheduler) | [F11-export.md](./F11-export.md) |
| F12 | Auth & Session (OIDC, Mock, Impersonation, Permissions) | [F12-auth-session.md](./F12-auth-session.md) |
| F13 | System-Infrastruktur (Feature Flags, Toggles, Scheduler, Clock, UUID) | [F13-system-infrastructure.md](./F13-system-infrastructure.md) |

## Verhältnis zu bestehenden Docs

Einige Features haben schon ältere, spezialisierte Docs im Verzeichnis
`docs/`:

- `absence-feature-frontend.md` → wird in `F05-absence-system.md` referenziert.
- `employee-management.md` / `_de.md` → wird in `F01-employee-management.md`
  referenziert und ergänzt.
- `block-report-templates/`, `template-examples/`, `test-examples/` → Referenz
  aus `F07-reporting-balance.md` bzw. `F10-templates-communication.md`.

Die neuen Feature-Dokumente sind **die verbindliche Referenz**. Die älteren
Dokumente werden schrittweise reingezogen.
