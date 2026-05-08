---
created: 2026-05-08T10:23:57.816Z
title: Cutover-UI als Admin-Feature (Frontend für /admin/cutover/*)
area: frontend
files:
  - rest/src/cutover.rs
  - service/src/cutover.rs
  - shifty-dioxus/src/page/
---

## Problem

Der Backend-Cutover (extra_hours → absence_period) ist nur via Swagger UI bedienbar.
Drei sequenzielle POST-Endpoints unter `/admin/cutover/*`:

- `POST /admin/cutover/profile` — Profile der Quelldaten (read-only)
- `POST /admin/cutover/gate-dry-run` — kompletter Dry-Run mit Tx-Rollback
- `POST /admin/cutover/commit` — atomic Tx: migration + carryover refresh + soft-delete + flag-flip

Für einen Nicht-Dev-Operator ist Swagger UI ungeeignet (kein State-Tracking zwischen
Stages, kein menschenlesbares Result-Display, keine Confirmation-Schleife vor dem
destruktiven `commit`). Das Feature ist als Backend bereits production-ready, fehlt
aber das Bedien-UI.

Aufgekommen während Phase 8 UAT-Setup (v1.3): User hat gefragt "wie starte ich die
Migration?", musste auf manuellen Swagger-Workflow verwiesen werden.

## Solution

Eigener Admin-Bereich im Frontend (Route z.B. `/admin/cutover`) mit:

- **Permission-Gate**: `cutover_admin`-Privileg → ohne 403/Redirect, nicht im TopBar sichtbar
- **Drei sichtbare Stages** (Wizard-Style oder 3-Spalten-Layout):
  1. **Profile** — Click → POST profile → Result-Card mit Quarantine-Counts pro Kategorie + zu migrierenden Row-Counts
  2. **Dry-Run** — disabled bis Profile grün → POST gate-dry-run → Result-Card mit `CutoverRunResultTO` (Counts pro Stage, Carryover-Diff per Employee)
  3. **Commit** — disabled bis Dry-Run grün → Confirmation-Dialog ("Diese Aktion ist destruktiv und unidempotent. Backup-Tabellen sind angelegt.") → POST commit → Result-Card + Erfolgs-Banner "Migration abgeschlossen, Feature-Flag geflippt"
- **Idempotenz-Hinweis**: Nach commit zeigt die Page einen "bereits durchgeführt"-Banner statt der drei Stages — der commit-Endpoint ist nicht idempotent und sollte nicht versehentlich erneut getriggert werden
- **Result-Display**: `CutoverRunResultTO` strukturiert rendern — Quarantine-Tabelle (failed-rows mit Begründung), Carryover-Diff-Tabelle (Employee × Year × old/new value), Feature-Flag-State

**Scope:** klein bis mittel (~1 Phase). Backend ist vollständig, nur FE-Composition.

**Trigger:** Frühestens wenn ein Nicht-Dev-Stakeholder den Cutover ausführen soll oder
wenn die Cutover-Logik bei mehreren Tenants/Datenbanken ausgeführt werden muss.
