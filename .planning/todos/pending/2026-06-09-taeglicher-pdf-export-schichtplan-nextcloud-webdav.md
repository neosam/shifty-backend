---
created: 2026-06-09 12:39
title: Täglicher PDF-Export der Wochen-Schichtpläne nach Nextcloud (WebDAV)
area: shiftplan / export
files:
  - service/src/ (neuer ExportService / ScheduledJob - TBD)
  - service_impl/src/reporting.rs (Schichtplan-Datenquelle?)
  - shifty_bin/src/main.rs (Job-Scheduling / DI)
---

## Problem

Es soll ein automatisierter Job existieren, der **einmal täglich** die
Schichtpläne der **kommenden Woche** als **PDF** generiert und das Ergebnis
per **WebDAV** in eine **konfigurierbare Nextcloud-Instanz** ablegt.

Hintergrund / Nutzen: Die Schichtpläne sollen ohne manuelles Zutun an einem
zentralen, für das Team zugänglichen Ort (Nextcloud) landen — z. B. zum
Aushang / Teilen, ohne dass jemand sich täglich ins System einloggen und
exportieren muss.

Offene Fragen, die vor dem Planen geklärt werden müssen:
- **Scheduling:** Wie wird der Job getriggert? Interner Scheduler im
  `shifty_bin` (z. B. tokio-cron / interval-Task) vs. externer Cron, der einen
  REST-Endpoint aufruft? Backend hat aktuell (Stand Kenntnis) keinen
  eingebauten Scheduler — das ist eine Architekturentscheidung.
- **PDF-Generierung:** Welche Crate/Ansatz? (HTML→PDF, direkte PDF-Lib,
  Template-Engine). Gibt es bereits eine Report-/Render-Schicht, an die das
  andocken kann? (`service_impl/src/reporting.rs` prüfen.)
- **"Nächste Woche":** Definition des Zeitraums (Mo–So der Folgewoche?
  rollierende 7 Tage?). Welche Schichtplan-Sicht/Aggregat liefert die Daten?
- **WebDAV-Client:** Welche Crate? Auth (App-Passwort / Basic-Auth über HTTPS).
- **Konfigurierbarkeit:** Nextcloud-URL, Zielordner, Credentials, Zeitpunkt des
  Laufs → über ENV/Config. Secrets-Handling beachten.
- **Mehrere Schichtpläne / Mandanten?** Ein PDF gesamt oder pro Schichtplan?
- **Fehlerbehandlung / Retry / Logging**, falls Nextcloud nicht erreichbar.

## Solution

TBD — Architekturentscheidung nötig (interner Scheduler vs. extern getriggerter
REST-Endpoint). Wahrscheinlich neuer Business-Logic-Service (z. B.
`ShiftplanExportService`), der bestehende Schichtplan-Read-Aggregate nutzt,
PDF rendert und über einen WebDAV-Client hochlädt. Konfiguration über ENV.

Eignet sich gut als eigene **Phase** (Multi-File, neue Dependencies, neue
Integration) — bei Aufgriff via `/gsd-add-phase` oder `/gsd-discuss-phase`
einsteigen, da mehrere Designfragen offen sind.
